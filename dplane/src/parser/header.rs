use crate::core::memory::array::Array;


pub struct Header {
    pub fields: Array<Field>,
    pub used_fields: Array<Field>,
    pub parse_fields: Array<Field>,
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
    pub fn new(field_len_list: &[u16], used_field_index_list: &[u16], parse_field_index_list: &[u16]) -> Self {
        let mut fields = Array::<Field>::new(field_len_list.len());
        let mut used_fields = Array::<Field>::new(used_field_index_list.len());
        let mut parse_fields = Array::<Field>::new(parse_field_index_list.len());
        
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

        for (i, field_index) in used_field_index_list.iter().enumerate() {
            used_fields[i] = fields[*field_index as usize].clone();
        }

        for (i, field_index) in parse_field_index_list.iter().enumerate() {
            parse_fields[i] = fields[*field_index as usize].clone();
        }


        Header {
            fields,
            used_fields,
            parse_fields,
            fields_len: field_len_list.len(),
            used_fields_len: used_field_index_list.len(),
        }
    }
}


impl Field {
    pub fn new(pre_field: &Field, field_bit_size: u16) -> Self {
        let mask_list = [128, 192, 224, 240, 248, 252, 254];
        let (start_byte_pos, start_bit_mask, readed_bit): (usize, u8, u16) = if (pre_field.end_bit_mask & 1) == 1 {
            if field_bit_size >= 8 {
                (pre_field.end_byte_pos + 1, 0xff, 8)
            } else {
                (pre_field.end_byte_pos+ 1, mask_list[field_bit_size as usize - 1], field_bit_size)
            }
        } else {
            let pre_end_bit_start_pos = if pre_field.end_bit_mask >= 128 {
                8
            } else if pre_field.end_bit_mask >= 64 {
                7
            } else if pre_field.end_bit_mask >= 32 {
                6
            } else if pre_field.end_bit_mask >= 16 {
                5
            } else if pre_field.end_bit_mask >= 8 {
                4
            } else if pre_field.end_bit_mask >= 4 {
                3
            } else if pre_field.end_bit_mask >= 2 {
                2
            } else if pre_field.end_bit_mask >= 1 {
                1
            } else {
                8
            };
            let pre_end_bit_count = (pre_field.end_bit_mask as u64).count_ones();

            let bit_space = pre_end_bit_start_pos - pre_end_bit_count;
            if bit_space > field_bit_size as u32 {
                (pre_field.end_byte_pos, mask_list[field_bit_size as usize - 1] >> ((8 - pre_end_bit_start_pos) + pre_end_bit_count), field_bit_size)
            } else {
                let xor_mask = [0xff, 0x7f, 0x3f, 0x1f, 0xf, 0x7, 0x3, 0x1];
                (pre_field.end_byte_pos, pre_field.end_bit_mask ^ xor_mask[(8 - pre_end_bit_start_pos) as usize], bit_space as u16)
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


    pub fn get(&self) -> (usize, u8, usize, u8) {
        (self.start_byte_pos, self.start_bit_mask, self.end_byte_pos, self.end_bit_mask)
    }


    pub fn copy_ptr_value(&self, base_offset: isize, src: *mut u8, dst: *mut u8) -> isize {
        unsafe {
            if self.start_byte_pos == self.end_byte_pos {
                *dst = *src.offset(base_offset + self.start_byte_pos as isize) & self.start_bit_mask;
                return 1;
            }
            *dst = *src.offset(base_offset + self.start_byte_pos as isize) & self.start_bit_mask;

            let mut dst_offset = 1;
            for i in (base_offset + self.start_byte_pos as isize + 1)..(base_offset+ self.end_byte_pos as isize) {
                *dst.offset(dst_offset) = *src.offset(i);
                dst_offset += 1;
            }


            *dst.offset(dst_offset) = *src.offset(base_offset + self.end_byte_pos as isize) & self.end_bit_mask;
            dst_offset += 1;
            dst_offset
        }
    }


    /**
     * pkt >= value
     */
    pub fn cmp_pkt_ge_value(&self, pkt: *const u8, hdr_offset: u16, value: &Array<u8>, end_bit_mask: u8) -> bool {
        // start
        if self.start_byte_pos == self.end_byte_pos {
            let pkt_first_value = unsafe {
                *(pkt.offset((self.start_byte_pos + hdr_offset as usize) as isize)) & self.start_bit_mask
            };

            if pkt_first_value >= value[0] {
                return true;
            }
            return false;

        }

        for i in 0..(value.len()-1) {
            let pkt_value = unsafe {
                *(pkt.offset((self.start_byte_pos + i + hdr_offset as usize) as isize))
            };

            if pkt_value > value[i] {
                return true;
            } else if pkt_value < value[i] {
                return false;
            }
        }


        // end
        let pkt_end_value = if (value.len() - 1) == (self.end_byte_pos - self.start_byte_pos) {
            unsafe {
                *(pkt.offset((self.end_byte_pos + hdr_offset as usize) as isize)) & end_bit_mask
            }
        } else {
            unsafe {
                *(pkt.offset((self.start_byte_pos + (value.len() - 1) + hdr_offset as usize) as isize)) & end_bit_mask
            }
        };

        if pkt_end_value >= (value[value.len() - 1] & end_bit_mask) {
            return true;
        }

        false
    }


    /**
     * pkt <= value
     */
    pub fn cmp_pkt_le_value(&self, pkt: *const u8, hdr_offset: u16, value: &Array<u8>, end_bit_mask: u8) -> bool {
        // start
        if self.start_byte_pos == self.end_byte_pos {
            let pkt_first_value = unsafe {
                *(pkt.offset((self.start_byte_pos + hdr_offset as usize) as isize)) & self.start_bit_mask
            };

            if pkt_first_value <= value[0] {
                return true;
            }
            return false;

        }

        for i in 0..(value.len()-1) {
            let pkt_value = unsafe {
                *(pkt.offset((self.start_byte_pos + i + hdr_offset as usize) as isize))
            };

            if pkt_value < value[i] {
                return true;
            } else if pkt_value > value[i] {
                return false;
            }
        }


        // end
        let pkt_end_value = if (value.len() - 1) == (self.end_byte_pos - self.start_byte_pos) {
            unsafe {
                *(pkt.offset((self.end_byte_pos + hdr_offset as usize) as isize)) & end_bit_mask
            }
        } else {
            unsafe {
                *(pkt.offset((self.start_byte_pos + (value.len() - 1) + hdr_offset as usize) as isize)) & end_bit_mask
            }
        };

        if pkt_end_value <= (value[value.len() - 1] & end_bit_mask) {
            return true;
        }

        false
    }


    pub fn cmp_pkt(&self, pkt: *const u8, hdr_offset: u16, value: &Array<u8>, end_bit_mask: u8) -> bool {
        // start
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


        // middle
        for i in 1..(value.len()-1) {
            let pkt_value = unsafe {
                *(pkt.offset((self.start_byte_pos + i + hdr_offset as usize) as isize))
            };

            if pkt_value != value[i] {
                return false;
            }
        }


        // end
        let pkt_end_value = if (value.len() - 1) == (self.end_byte_pos - self.start_byte_pos) {
            unsafe {
                *(pkt.offset((self.end_byte_pos + hdr_offset as usize) as isize)) & end_bit_mask
            }
        } else {
            unsafe {
                *(pkt.offset((self.start_byte_pos + (value.len() - 1) + hdr_offset as usize) as isize)) & end_bit_mask
            }
        };

        if pkt_end_value != value[value.len() - 1] & end_bit_mask {
            return false;
        }

        true
    }
}


#[cfg(test)]
mod tests {
    use super::Field;
    use crate::core::memory::array::Array;

    #[test]
    fn test_field_new() {
        let mut field = Field {
            start_byte_pos: 0,
            start_bit_mask: 0,
            end_byte_pos: 0,
            end_bit_mask: 0,
        };

        field = Field::new(&field, 48);
        assert_eq!(field.start_byte_pos, 0);
        assert_eq!(field.start_bit_mask, 0xff);
        assert_eq!(field.end_byte_pos, 5);
        assert_eq!(field.end_bit_mask, 0xff);

        field = Field::new(&field, 48);
        assert_eq!(field.start_byte_pos, 6);
        assert_eq!(field.start_bit_mask, 0xff);
        assert_eq!(field.end_byte_pos, 11);
        assert_eq!(field.end_bit_mask, 0xff);

        field = Field::new(&field, 8);
        assert_eq!(field.start_byte_pos, 12);
        assert_eq!(field.start_bit_mask, 0xff);
        assert_eq!(field.end_byte_pos, 12);
        assert_eq!(field.end_bit_mask, 0xff);

        field = Field::new(&field, 1);
        assert_eq!(field.start_byte_pos, 13);
        assert_eq!(field.start_bit_mask, 0x80);
        assert_eq!(field.end_byte_pos, 13);
        assert_eq!(field.end_bit_mask, 0x80);

        field = Field::new(&field, 2);
        assert_eq!(field.start_byte_pos, 13);
        assert_eq!(field.start_bit_mask, 0x60);
        assert_eq!(field.end_byte_pos, 13);
        assert_eq!(field.end_bit_mask, 0x60);

        field = Field::new(&field, 16);
        assert_eq!(field.start_byte_pos, 13);
        assert_eq!(field.start_bit_mask, 0x1F);
        assert_eq!(field.end_byte_pos, 15);
        assert_eq!(field.end_bit_mask, 0xE0);

        field = Field::new(&field, 2);
        assert_eq!(field.start_byte_pos, 15);
        assert_eq!(field.start_bit_mask, 0x18);
        assert_eq!(field.end_byte_pos, 15);
        assert_eq!(field.end_bit_mask, 0x18);

        field = Field::new(&field, 3);
        assert_eq!(field.start_byte_pos, 15);
        assert_eq!(field.start_bit_mask, 0x7);
        assert_eq!(field.end_byte_pos, 15);
        assert_eq!(field.end_bit_mask, 0x7);

        field = Field::new(&field, 8);
        assert_eq!(field.start_byte_pos, 16);
        assert_eq!(field.start_bit_mask, 0xff);
        assert_eq!(field.end_byte_pos, 16);
        assert_eq!(field.end_bit_mask, 0xff);
    }


    #[test]
    fn test_copy_ptr_value() {
        let mut src = Array::<u8>::new(10);
        let mut dst = Array::<u8>::new(10);

        src[0] = 10;
        src[1] = 20;
        src[2] = 30;
        let mut field = Field {
            start_byte_pos: 0,
            start_bit_mask: 0xff,
            end_byte_pos: 2,
            end_bit_mask: 0xff,
        }; 
        let mut next = field.copy_ptr_value(0, src.as_ptr(), dst.as_ptr());
        assert_eq!(src[0], dst[0]);
        assert_eq!(src[1], dst[1]);
        assert_eq!(src[2], dst[2]);
        assert_eq!(next, 3);


        src[3] = 40;
        src[4] = 50;
        src[5] = 60;
        field = Field {
            start_byte_pos: 0,
            start_bit_mask: 0xff,
            end_byte_pos: 2,
            end_bit_mask: 0xff,
        }; 
        next = field.copy_ptr_value(3, src.as_ptr(), unsafe { dst.as_ptr().offset(3) });
        assert_eq!(src[3], dst[3]);
        assert_eq!(src[4], dst[4]);
        assert_eq!(src[5], dst[5]);
        assert_eq!(next, 3);


        src[6] = 70;
        src[7] = 129;
        field = Field {
            start_byte_pos: 6,
            start_bit_mask: 0xff,
            end_byte_pos: 7,
            end_bit_mask: 0x80,
        }; 
        next = field.copy_ptr_value(0, src.as_ptr(), unsafe { dst.as_ptr().offset(6) });
        assert_eq!(src[6], dst[6]);
        assert_eq!(128, dst[7]);
        assert_eq!(next, 2);
    }


    #[test]
    fn test_cmp_pkt() {
        let mut pkt = Array::<u8>::new(10);
        pkt[0] = 10;
        pkt[1] = 20;
        pkt[2] = 129;
        pkt[3] = 40;
        pkt[4] = 50;
        pkt[5] = 60;
        pkt[6] = 70;
        pkt[7] = 80;
        pkt[8] = 64;
        pkt[9] = 100;


        let mut field = Field {
            start_byte_pos: 0,
            start_bit_mask: 0xff,
            end_byte_pos: 2,
            end_bit_mask: 0x80,
        }; 
        let mut value = Array::<u8>::new(3);
        value[0] = 10;
        value[1] = 20;
        value[2] = 128;
        assert!(field.cmp_pkt(pkt.as_ptr(), 0, &value, 0x80));
        value.free();


        // LPM
        field = Field {
            start_byte_pos: 0,
            start_bit_mask: 0xff,
            end_byte_pos: 3,
            end_bit_mask: 0xff,
        }; 
        value = Array::<u8>::new(3);
        value[0] = 10;
        value[1] = 20;
        value[2] = 129;
        assert!(field.cmp_pkt(pkt.as_ptr(), 0, &value, 0xff));
        value.free();


        // LPM
        field = Field {
            start_byte_pos: 0,
            start_bit_mask: 0xff,
            end_byte_pos: 3,
            end_bit_mask: 0xff,
        }; 
        value = Array::<u8>::new(4);
        value[0] = 10;
        value[1] = 20;
        value[2] = 129;
        value[3] = 32;
        assert!(field.cmp_pkt(pkt.as_ptr(), 0, &value, 0xE0));
        value.free();


        field = Field {
            start_byte_pos: 0,
            start_bit_mask: 0xff,
            end_byte_pos: 3,
            end_bit_mask: 0xff,
        }; 
        value = Array::<u8>::new(4);
        value[0] = 40;
        value[1] = 50;
        value[2] = 60;
        value[3] = 70;
        assert!(field.cmp_pkt(pkt.as_ptr(), 3, &value, 0xff));
        value.free();


        field = Field {
            start_byte_pos: 0,
            start_bit_mask: 0xC0,
            end_byte_pos: 0,
            end_bit_mask: 0xC0,
        }; 
        value = Array::<u8>::new(1);
        value[0] = 64;
        assert!(field.cmp_pkt(pkt.as_ptr(), 8, &value, 0xC0));
        value.free();
    }


    #[test]
    fn test_cmp_pkt_ge_value() {
        let mut pkt = Array::<u8>::new(10);
        pkt[0] = 10;
        pkt[1] = 20;
        pkt[2] = 30;


        let mut field = Field {
            start_byte_pos: 0,
            start_bit_mask: 0xff,
            end_byte_pos: 2,
            end_bit_mask: 0xff,
        }; 
        let mut value = Array::<u8>::new(3);
        value[0] = 10;
        value[1] = 20;
        value[2] = 30;
        assert!(field.cmp_pkt_ge_value(pkt.as_ptr(), 0, &value, 0xff));

        value[0] = 10;
        value[1] = 20;
        value[2] = 31;
        assert!(!field.cmp_pkt_ge_value(pkt.as_ptr(), 0, &value, 0xff));

        value[0] = 10;
        value[1] = 20;
        value[2] = 29;
        assert!(field.cmp_pkt_ge_value(pkt.as_ptr(), 0, &value, 0xff));

        value[0] = 20;
        value[1] = 20;
        value[2] = 30;
        assert!(!field.cmp_pkt_ge_value(pkt.as_ptr(), 0, &value, 0xff));

        pkt[0] = 0;
        pkt[1] = 0;
        pkt[2] = 30;

        value[0] = 0;
        value[1] = 0;
        value[2] = 30;
        assert!(field.cmp_pkt_ge_value(pkt.as_ptr(), 0, &value, 0xff));

        value[0] = 0;
        value[1] = 0;
        value[2] = 29;
        assert!(field.cmp_pkt_ge_value(pkt.as_ptr(), 0, &value, 0xff));
    }


    #[test]
    fn test_cmp_pkt_le_value() {
        let mut pkt = Array::<u8>::new(10);
        pkt[0] = 10;
        pkt[1] = 20;
        pkt[2] = 30;


        let mut field = Field {
            start_byte_pos: 0,
            start_bit_mask: 0xff,
            end_byte_pos: 2,
            end_bit_mask: 0xff,
        }; 
        let mut value = Array::<u8>::new(3);
        value[0] = 10;
        value[1] = 20;
        value[2] = 30;
        assert!(field.cmp_pkt_le_value(pkt.as_ptr(), 0, &value, 0xff));

        value[0] = 10;
        value[1] = 20;
        value[2] = 31;
        assert!(field.cmp_pkt_le_value(pkt.as_ptr(), 0, &value, 0xff));

        value[0] = 10;
        value[1] = 20;
        value[2] = 29;
        assert!(!field.cmp_pkt_le_value(pkt.as_ptr(), 0, &value, 0xff));

        value[0] = 20;
        value[1] = 20;
        value[2] = 30;
        assert!(field.cmp_pkt_le_value(pkt.as_ptr(), 0, &value, 0xff));

        pkt[0] = 0;
        pkt[1] = 0;
        pkt[2] = 30;

        value[0] = 0;
        value[1] = 0;
        value[2] = 30;
        assert!(field.cmp_pkt_le_value(pkt.as_ptr(), 0, &value, 0xff));

        value[0] = 0;
        value[1] = 0;
        value[2] = 29;
        assert!(!field.cmp_pkt_le_value(pkt.as_ptr(), 0, &value, 0xff));
    }
}
