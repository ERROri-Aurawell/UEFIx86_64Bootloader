use uefi::Identify;
use uefi::boot::{
    OpenProtocolAttributes, OpenProtocolParams, SearchType, image_handle, locate_handle_buffer,
    open_protocol,
};

use uefi::proto::media::block::BlockIO;
use uefi::{Error, Status};

extern crate alloc;
use alloc::vec::Vec;

use log::info;

/// Tipos de drive para filtragem de busca.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveType {
    /// Disco físico principal (não é partição lógica).
    PhysicalDisk,
    /// Partição lógica (ex: GPT/MBR partition).
    Partition,
    /// Mídia removível (USB, CD-ROM, etc.).
    Removable,
    /// Qualquer dispositivo de bloco disponível.
    Any,
}

/// Lê dados brutos de um dispositivo de bloco baseado no tipo especificado.
pub fn read_raw_drive(drive_type: DriveType, lba: u64, buffer: &mut Vec<u8>) -> Result<(), Error> {
    info!("Lendo dados brutos...");

    // 1. Localiza todos os handles que suportam o protocolo BlockIo
    let handles = locate_handle_buffer(SearchType::ByProtocol(&BlockIO::GUID))?;
    info!("Total de handles de bloco encontrados: {}", handles.len());

    for (idx, handle) in handles.iter().enumerate() {
        info!("Analisando handle [{}]...", idx);

        // 2. Abertura não-exclusiva compatível com uefi 0.39.0
        let mut block_io = match unsafe {
            open_protocol::<BlockIO>(
                OpenProtocolParams {
                    handle: *handle,
                    agent: image_handle(),
                    controller: None,
                },
                OpenProtocolAttributes::GetProtocol,
            )
        } {
            Ok(io) => io,
            Err(e) => {
                info!(
                    "Handle [{}]: falha ao abrir protocolo: {:?}",
                    idx,
                    e.status()
                );
                continue;
            }
        };

        let media = block_io.media();

        // 3. Ignora mídias não presentes (ex: leitores de CD/Disquete vazios no QEMU)
        if !media.is_media_present() {
            info!("Handle [{}]: mídia não presente", idx);
            continue;
        }

        // 4. Aplica o filtro de tipo de drive
        let is_match = match drive_type {
            DriveType::PhysicalDisk => !media.is_logical_partition(),
            DriveType::Partition => media.is_logical_partition(),
            DriveType::Removable => media.is_removable_media(),
            DriveType::Any => true,
        };

        if !is_match {
            info!("Handle [{}]: filtro {:?} não bateu", idx, drive_type);
            continue;
        }

        info!("Handle [{}]: dispositivo correspondente encontrado!", idx);

        // 5. Validação do tamanho do bloco e do buffer
        let block_size = media.block_size() as usize;
        if block_size == 0 || buffer.is_empty() || buffer.len() % block_size != 0 {
            info!(
                "Handle [{}]: buffer inválido (len: {}, block_size: {})",
                idx,
                buffer.len(),
                block_size
            );
            return Err(Error::new(Status::BAD_BUFFER_SIZE, ()));
        }

        // 6. Validação dos limites de LBA
        let blocks_to_read = (buffer.len() / block_size) as u64;
        let last_lba = lba
            .checked_add(blocks_to_read - 1)
            .ok_or_else(|| Error::new(Status::INVALID_PARAMETER, ()))?;

        if last_lba > media.last_block() {
            info!(
                "Handle [{}]: LBA {} fora do limite máximo {}",
                idx,
                last_lba,
                media.last_block()
            );
            return Err(Error::new(Status::INVALID_PARAMETER, ()));
        }

        // 7. CORREÇÃO CRÍTICA: Alinhamento de Memória para DMA (IoAlign)
        let io_align = media.io_align() as usize;
        let requires_alignment = io_align > 1 && ((buffer.as_ptr() as usize) % io_align != 0);

        if requires_alignment {
            info!(
                "Handle [{}]: Requer alinhamento DMA para {} bytes. Criando buffer alinhado...",
                idx, io_align
            );

            let total_size = buffer.len();
            let mut temp_buf = alloc::vec![0u8; total_size + io_align];
            let ptr_val = temp_buf.as_ptr() as usize;
            let offset = if ptr_val % io_align == 0 {
                0
            } else {
                io_align - (ptr_val % io_align)
            };

            let aligned_slice = &mut temp_buf[offset..offset + total_size];

            info!(
                "Handle [{}]: Chamando read_blocks com buffer alinhado...",
                idx
            );
            block_io.read_blocks(media.media_id(), lba, aligned_slice)?;
            buffer.copy_from_slice(aligned_slice);
        } else {
            info!("Handle [{}]: Chamando read_blocks...", idx);
            block_io.read_blocks(media.media_id(), lba, buffer)?;
        }

        info!("Handle [{}]: Leitura realizada com sucesso!", idx);
        return Ok(());
    }

    info!("Nenhum dispositivo correspondente foi localizado.");
    Err(Error::new(Status::NOT_FOUND, ()))
}
