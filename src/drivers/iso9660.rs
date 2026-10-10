use crate::drivers::controller::{File, FileType, Find};
use crate::drivers::raw::{DriveType, read_raw_drive};

use uefi::Error;

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use log::info;

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
    fn find_file(&self, file_name: &str, ft: FileType) -> Result<File, Error> {
        info!("Searching from: {}", &file_name);

        let mut primary_descriptor: Vec<u8> = vec![0u8; 0x800];
        info!("Espaço para o PVD Alocado");

        //LBA 0 - 15 System Area  (FAT 32 - El Torito)

        //LBA 16 - Primary Volume Descriptor
        read_raw_drive(DriveType::Removable, 16, &mut primary_descriptor)?;

        info!(
            "itz a valid ISO? : {}",
            ISO9660::validate_pvd(&primary_descriptor)
        );

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

*/
