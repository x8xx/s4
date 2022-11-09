use crate::core::memory::array::Array;

pub struct Header {
    pub fields: Array<Field>,
    pub used_fields: Array<Field>,
    pub fields_len: usize,
    pub used_fields_len: usize,
}

#[derive(Clone, Copy)]
pub struct Field {
    pub start_byte_pos: usize,
    pub start_bit_mask: u8,
    pub end_byte_pos: usize,
    pub end_bit_mask: u8,
}

impl Header {
    pub fn new(field_len_list: &[u16], used_field_len_list: &[u16]) -> Self {
        let mut fields = Array::<Field>::new(field_len_list.len());
        let mut used_fields = Array::<Field>::new(used_field_len_list.len());
        
        let mut pre_field = &Field {
            start_byte_pos: 0,
            start_bit_mask: 0,
            end_byte_pos: 0,
            end_bit_mask: 0,
        };

        for (i, field_bit_size) in field_len_list.iter().enumerate() {
            fields[i] = Field::new(&pre_field, *field_bit_size);
            pre_field = &fields[i];
        } 

        for (i, field_index) in used_field_len_list.iter().enumerate() {
            used_fields[i] = fields[*field_index as usize].clone();
        }

        Header {
            fields,
            used_fields,
            fields_len: field_len_list.len(),
            used_fields_len: used_field_len_list.len(),
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


    pub fn cmp_pkt(&self, pkt: *const u8, hdr_offset: u16, value: &Array<u8>, end_bit_mask: u8) -> bool {
        if self.start_byte_pos == self.end_byte_pos {
            let pkt_first_value = unsafe {
                *(pkt.offset((self.start_byte_pos + hdr_offset as usize) as isize)) & self.start_bit_mask
            };

            if pkt_first_value != value[0] {
                return false;
            }
            return true;

        } else {
            let pkt_first_value = unsafe {
                *(pkt.offset((self.start_byte_pos + hdr_offset as usize) as isize))
            };

            if pkt_first_value != value[0] {
                return false;
            }
        }

        for i in (self.start_byte_pos + 1)..self.end_byte_pos {
            let pkt_value = unsafe {
                *(pkt.offset((i + hdr_offset as usize) as isize))
            };

            if pkt_value != value[i - self.start_byte_pos] {
                return false;
            }

        }

        let pkt_end_value = unsafe {
            *(pkt.offset((self.end_byte_pos + hdr_offset as usize) as isize)) & self.end_bit_mask
        };

        if pkt_end_value != value[value.len() - 1] & end_bit_mask {
            return false;
        }
        true
    }


//     pub fn cmp_exact_match(&self, pkt: *const u8, value: &Array<u8>, offset: u16) -> bool {
//         true
//     }

//     pub fn cmp_lpm_match(&self, pkt: *const u8, value: &Array<u8>, offset: u16, prefix: u8) -> bool {
//         true
//     }
}
