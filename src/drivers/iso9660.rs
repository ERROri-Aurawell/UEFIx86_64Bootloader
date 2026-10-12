use core::error::Error;

extern crate alloc;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;

use crate::drivers::controller::{ErrorType, File, FileError, FileType, Find};
use crate::drivers::raw::{DriveType, read_raw_drive};

use log::info;

//Não precisava, com certeza absoluta
struct IsoData {
    pub t_code:             u8,        // 0x01
    pub std_ident:          [u8 ; 5  ],// "CD001"
    pub version:            u8,        // 0x01
                                       // Unused 0x00
    pub sys_ident:          [u8 ; 32 ],// 0x00 - 0x0F
    pub vol_ident:          [u8 ; 32 ],// Volume Name
                                       // [0x00 ; 8]
    pub vol_space_size:     i32,       // N of Logical Blocks
                                       // [0x00 ; 32]
    pub vol_set_size:       i16,       // Size of set (n of disks)
    pub vol_seq_num:        i16,       // N of this disk
    pub logic_block_size:   i16,       // Size of logical block (i'm cooked...)
    pub path_table_size:    i32,       // Size of path table
    pub loc_type_l_table:   i32,       // LBA loc little-endian
    pub loc_opt_type_l:     i32,       // optional LBA loc, 0x00 or path, little-endian
    pub loc_type_m_table:   i32,       // LBA loc path table - big-endian
    pub loc_opt_type_m:     i32,       // optional LBA loc, 0x00 or path, big-endian
    pub root_dir_entry:     [u8 ; 34 ],// Directory entry for the root directory	-	Note that this is not an LBA address, it is the actual Directory Record, which contains a single byte Directory Identifier (0x00), hence the fixed 34 byte size.
    pub vol_set_ident:      [u8 ; 128],// Identifier of the volume set of which this volume is a member.
    pub publisher_ident:    [u8 ; 128],// The volume publisher. For extended publisher information, the first byte should be 0x5F, followed by the filename of a file in the root directory. If not specified, all bytes should be 0x20.
    pub data_prep_ident:    [u8 ; 128],// The identifier of the person(s) who prepared the data for this volume. For extended preparation information, the first byte should be 0x5F, followed by the filename of a file in the root directory. If not specified, all bytes should be 0x20.
    pub application_ident:  [u8 ; 128],// Identifies how the data are recorded on this volume. For extended information, the first byte should be 0x5F, followed by the filename of a file in the root directory. If not specified, all bytes should be 0x20.
    pub copyright_ident:    [u8 ; 37 ],// Filename of a file in the root directory that contains copyright information for this volume set. If not specified, all bytes should be 0x20.
    pub abstract_ident:     [u8 ; 37 ],// Filename of a file in the root directory that contains abstract information for this volume set. If not specified, all bytes should be 0x20.
    pub bib_ident:          [u8 ; 37 ],// Filename of a file in the root directory that contains bibliographic information for this volume set. If not specified, all bytes should be 0x20.
    //pub vol_creation_date:  u8,        // Datetime - The date and time of when the volume was created.
    //pub vol_mod_date:       u8,        // Datetime - The date and time of when the volume was modified.
    //pub vol_exp_date:       u8,        // Datetime - The date and time after which this volume is considered to be obsolete. If not specified, then the volume is never considered to be obsolete.
    //pub vol_effec_date:     u8,        // Datetime - The date and time after which the volume may be used. If not specified, the volume may be used immediately.
    pub file_struct_ver:    u8,        // 0x01.
                                       // [0x00 ; 512]
    pub reserved:           [u8 ; 653],// Reserved [0x?? ; 653]
}
pub struct ISO9660 {}

impl ISO9660 {
    fn validate_pvd(buffer: &Vec<u8>) -> bool {
        //0x01
        let is_pvd_type = buffer[0] == 0x01;

        //strA CD001
        let is_cd001 = &buffer[1..6] == b"CD001";

        //0x01
        let is_version_1 = buffer[6] == 0x01;

        //Zero
        let literal_nothing = buffer[7] == 0x00;

        is_pvd_type && is_cd001 && is_version_1 && literal_nothing
    }
}

impl Find for ISO9660 {
    fn find_file(&self, file_name: &str, ft: FileType) -> Result<File, Box<dyn Error>> {
        info!("Searching from: {}", &file_name);

        let mut primary_descriptor: Vec<u8> = vec![0u8; 0x800];
        info!("Espaço para o PVD Alocado");

        //LBA 0 - 15 System Area 

        //LBA 16 - Primary Volume Descriptor
        read_raw_drive(DriveType::Removable, 16, &mut primary_descriptor)?;


        //Erro customizado
        if !ISO9660::validate_pvd(&primary_descriptor) {
            return Err(Box::new(FileError {
                error: ErrorType::InvalidPvd,
            }));
        }

        //Sessão que faria qualquer dev rust sério querer me matar : 


        loop {}
    }
}

/*
    |-----------------------------------------------------------------------------|
    |                                 Sector size                                 |
    |           -- An ISO 9660 sector is normally 2 KiB long                      |
    |-----------------------------------------------------------------------------|

    |-----------------------------------------------------------------------------|
    |                               Numerical formats                             |
    |-----------------------------------------------------------------------------|
    |   Encoding        |    Description                                          |
    |-----------------------------------------------------------------------------|
    |   int8            |    u8.                                                  |
    |-----------------------------------------------------------------------------|
    |   sint8	        |    i8.                                                  |
    |-----------------------------------------------------------------------------|
    |   int16_LSB       |    Little-endian encoded u16.                           |
    |-----------------------------------------------------------------------------|
    |   int16_MSB       |    Big-endian encoded u16.                              |
    |-----------------------------------------------------------------------------|
    |   int16_LSB-MSB	|   Little-endian followed by big-endian encoded u16.     |
    |-----------------------------------------------------------------------------|
    |   sint16_LSB	    |   Little-endian encoded i16.                            |
    |-----------------------------------------------------------------------------|
    |   sint16_MSB	    |   Big-endian encoded i16.                               |
    |-----------------------------------------------------------------------------|
    |   sint16_LSB-MSB	|   Little-endian followed by big-endian encoded i16.     |
    |-----------------------------------------------------------------------------|
    |   int32_LSB	    |   Little-endian encoded u32.                            |
    |-----------------------------------------------------------------------------|
    |   int32_MSB	    |   Big-endian encoded u32.                               |
    |-----------------------------------------------------------------------------|
    |   int32_LSB-MSB	|   Little-endian followed by big-endian encoded u32.     |
    |-----------------------------------------------------------------------------|
    |   sint32_LSB	    |   Little-endian encoded i32.                            |
    |-----------------------------------------------------------------------------|
    |   sint32_MSB	    |   Big-endian encoded i32.                               |
    |-----------------------------------------------------------------------------|
    |   sint32_LSB-MSB	|   Little-endian followed by big-endian encoded i32.     |
    |-----------------------------------------------------------------------------|

    |------------------------------------------------------------------------------------------|
    |              String format                                                               |
    |------------------------------------------------------------------------------------------|
    |   Character strings are encoded with ASCII encoding.                                     |
    |   The specification does not permit all characters.                                      |
    |   It defines two sets of characters:                                                     |
    |                                                                                          |
    |           'a-characters'                                                                 |
    |           'd-characters'                                                                 |
    |                                                                                          |
    |   You will see these terms used in the descriptor tables throughout this article.        |
    |   The character sets are:                                                                |
    |                                                                                          |
    |   a-characters:                                                                          |
    |       A B C D E F G H I J K L M N O P                                                    |
    |       Q R S T U V W X Y Z 0 1 2 3 4 5                                                    |
    |       6 7 8 9 _ ! " % & ' ( ) * + , -                                                    |
    |       . / : ; < = > ?                                                                    |
    |                                                                                          |
    |   d-characters:                                                                          |
    |       A B C D E F G H I J K L M N O P                                                    |
    |       Q R S T U V W X Y Z 0 1 2 3 4 5                                                    |
    |       6 7 8 9 _                                                                          |
    |                                                                                          |
    |------------------------------------------------------------------------------------------|
    |    Encoding  |    Description                                                            |
    |------------------------------------------------------------------------------------------|
    |   strA	   |    String with only ASCII a-characters, padded to the right with spaces.  |
    |   strD       |	String with only ASCII d-characters, padded to the right with spaces.  |
    |------------------------------------------------------------------------------------------|

    
    |-------------------------------------------------------------------------------------------------------|
    |                                           Volume Descriptors                                          |
    |-------------------------------------------------------------------------------------------------------|
    |   Offset  | Length (bytes) |   Field name  |  Datatype    |	Description                             |
    |-------------------------------------------------------------------------------------------------------|
    |   0	    |   1	         |  Type	     |  int8	    |   Volume Descriptor type code             |
    |   1       |   5	         |  Identifier	 |  strA	    |   Always 'CD001'                          |
    |   6	    |   1	         |  Version	     |  int8	    |   Volume Descriptor Version (0x01)        |
    |   7	    |   2041	     |  Data		 |  -           |   Depends on the volume descriptor type   |
    |-------------------------------------------------------------------------------------------------------|


    |----------------------------------------------------|
    |           Volume Descriptor Type Codes             |
    |----------------------------------------------------|
    |   Value	 |    Description                        |
    |----------------------------------------------------|
    |   0	     |    Boot Record                        |
    |   1	     |    Primary Volume Descriptor          |
    |   2	     |    Supplementary Volume Descriptor    |
    |   3	     |    Volume Partition Descriptor        |
    |   4-254	 |    Reserved                           |
    |   255	     |    Volume Descriptor Set Terminator   |
    |----------------------------------------------------|

    |-------------------------------------------------------------------------------------------|
    |               DateTime                                                                    |
    |-------------------------------------------------------------------------------------------|
    |  Offset  | Size | Description                                                             |
    |-------------------------------------------------------------------------------------------|
    |    0     | 1    | Number of years since 1900.                                             |
    |    1     | 1    | Month of the year from 1 to 12.                                         |
    |    2     | 1    | Day of the month from 1 to 31.                                          |
    |    3     | 1    | Hour of the day from 0 to 23.                                           |
    |    4     | 1    | Minute of the hour from 0 to 59.                                        |
    |    5     | 1    | Second of the minute from 0 to 59.                                      |
    |    6     | 1    | Offset from GMT in 15 minute intervals from -48 (West) to +52 (East)    |
    |-------------------------------------------------------------------------------------------|

*/
