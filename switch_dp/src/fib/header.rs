use crate::dpdk::dpdk_memory;

pub struct ParsedHeader {
    pub hdrs: *mut (*mut Header, u8),
    pub size: isize,
}

pub struct Header {
    pub fields: *const Field,
    pub used_fields: *const Field,
}

#[derive(Clone, Copy)]
pub struct Field {
    pub start_byte_pos: usize,
    pub start_bit_mask: u8,
    pub end_byte_pos: usize,
    pub end_bit_mask: u8,
}

impl Header {
    pub fn new(name: String, raw_fields: &[u16], raw_used_fields: &[u16]) -> Self {
        let fields_name = format!("{}_fields", name);
        let used_fields_name = format!("{}_used_fields", name);
        let fields = dpdk_memory::malloc::<Field>(fields_name, raw_fields.len() as u32);
        let used_fields = dpdk_memory::malloc::<Field>(used_fields_name, raw_used_fields.len() as u32);
        
        let mut pre_field = Field {
            start_byte_pos: 0,
            start_bit_mask: 0,
            end_byte_pos: 0,
            end_bit_mask: 0,
        };
        for (i, field_bit_size) in raw_fields.iter().enumerate() {
            unsafe {
                *fields.offset(i as isize) = Field::new(&pre_field, *field_bit_size);
                pre_field = *fields.offset(i as isize);
            }
        } 

        for (i, field_index) in raw_used_fields.iter().enumerate() {
            unsafe {
                *used_fields.offset(i as isize) = *fields.offset(*field_index as isize);
            }
        }

        Header {
            fields,
            used_fields,
        }
    }
}


impl Field {
    pub fn new(pre_field: &Field, field_bit_size: u16) -> Self {
        let mask_list = [128, 192, 224, 240, 248, 252, 254];
        let (start_byte_pos, start_bit_mask, readed_bit): (usize, u8, u16) = if pre_field.end_bit_mask == 0xff {
            if field_bit_size >= 8 {
                (pre_field.end_byte_pos + 1, 0xff, 8)
            } else {
                (pre_field.end_byte_pos+ 1, mask_list[field_bit_size as usize - 1], field_bit_size)
            }
        } else {
            let pre_end_bit_count = (pre_field.end_bit_mask as u64).count_ones();
            let bit_space = 8 - pre_end_bit_count;
            if bit_space > field_bit_size as u32 {
                (pre_field.end_byte_pos, mask_list[field_bit_size as usize - 1] >> pre_end_bit_count, field_bit_size)
            } else {
                (pre_field.end_byte_pos, pre_field.end_bit_mask ^ 0xff, bit_space as u16)
            }
        };

        let field_bit_size = field_bit_size - readed_bit;
        let (end_byte_pos, end_bit_mask): (usize, u8) = if field_bit_size == 0 {
            (start_byte_pos, start_bit_mask)
        } else {
            let residue_bit = field_bit_size % 8;
            if residue_bit == 0 {
                (start_byte_pos + ((field_bit_size / 8) as usize), 0xff)
            } else {
                (start_byte_pos + ((field_bit_size / 8) as usize) + 1, mask_list[residue_bit as usize - 1])
            }
        };

        Field {
            start_byte_pos,
            start_bit_mask,
            end_byte_pos,
            end_bit_mask,
        }

    }
}
