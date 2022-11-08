use std::ffi::c_void;
use std::mem::transmute;
use crate::core::memory::ring::Ring;
use crate::core::memory::ring::RingBuf;
use crate::core::memory::array::Array;
use crate::core::network::pktbuf::PktBuf;
use crate::core::network::interface::Interface;
use crate::parser::parser::Parser;
use crate::parser::parse_result::ParseResult;

#[repr(C)]
pub struct RxArgs<'a> {
    pub name: String,
    pub parser: Parser,
    pub header_list_len: usize,
    pub batch_count: usize,
    pub pktbuf_len: usize,
    pub pipeline_ring_list: &'a Array<Ring>,
}

pub struct RxResult<'a> {
    pub owner_ring: &'a RingBuf<RxResult<'a>>,
    pub id: usize,
    pub pktbuf: &'a mut PktBuf,
    pub raw_pkt: *mut u8,
    pub parse_result: ParseResult,
}

impl<'a> RxResult<'a> {
    pub fn free(&mut self) {
        self.pktbuf.free();
        self.owner_ring.free(self);
    }
}

pub extern "C" fn start_rx(rx_args_ptr: *mut c_void) -> i32 {
    println!("Start Rx Core");
    let rx_args = unsafe { &mut *transmute::<*mut c_void, *mut RxArgs>(rx_args_ptr) };

    let rx_result_ring_buf = RingBuf::new(rx_args.pktbuf_len);
    let mut pktbuf_list = Array::<PktBuf>::new(rx_args.pktbuf_len);

    // initialize rx_result_ring_buf object
    {
        let rx_result_array = Array::<&mut RxResult>::new(rx_args.pktbuf_len);
        rx_result_ring_buf.malloc_bulk(rx_result_array.as_slice(), rx_result_array.len());
        for (i, rx_result) in rx_result_array.as_slice().iter_mut().enumerate() {
            rx_result.owner_ring = &rx_result_ring_buf;
            rx_result.pktbuf = pktbuf_list.get(i);
            rx_result.parse_result.header_list = Array::new(rx_args.header_list_len);
        }
        rx_result_ring_buf.free_bulk(rx_result_array.as_slice(), rx_result_ring_buf.len());
        rx_result_array.free();
    }


    let interface = Interface::new(&rx_args.name);
    let mut next_pipeline_core = 0;
    let mut next_pktbuf_index = 0;
    let rx_result_ptrs_for_reset = Array::<&mut RxResult>::new(rx_args.batch_count);
    let mut count = 0;
    loop {
        let pkt_count = interface.rx(&mut pktbuf_list[next_pktbuf_index], rx_args.batch_count);
        for i in 0..pkt_count as usize {
            count += 1;
            // println!("count {}, pkt {}", count, pkt_count);
            // println!("malloc");
            let rx_result = rx_result_ring_buf.malloc();
            rx_result.id = count;
            // println!("malloc ok");
            let pktbuf = &rx_result.pktbuf;
            println!("test5 {} {}", i, next_pktbuf_index);
            let (pkt, pkt_len) = pktbuf.get_raw_pkt();
            rx_result.raw_pkt = pkt;
            if  !rx_args.parser.parse(pkt, pkt_len, &mut rx_result.parse_result) {
                continue;
            }

            rx_args.pipeline_ring_list[next_pipeline_core].enqueue(rx_result);
            next_pipeline_core += 1;

            // if next_pipeline_core == rx_args.pipeline_ring_list {
            //     next_pipeline_core = 0;
            // }
            next_pipeline_core &= rx_args.pipeline_ring_list.len();
        }

        next_pktbuf_index += pkt_count as usize;
        if (next_pktbuf_index + rx_args.batch_count) >= pktbuf_list.len() {
            let next_pktbuf_free_space = pktbuf_list.len() - next_pktbuf_index;
            println!("reset buf index {}, {}, {}", count, next_pktbuf_index, next_pktbuf_free_space);
            rx_result_ring_buf.malloc_bulk(rx_result_ptrs_for_reset.as_slice(), next_pktbuf_free_space);
            rx_result_ring_buf.free_bulk(rx_result_ptrs_for_reset.as_slice(), next_pktbuf_free_space);
            next_pktbuf_index = 0;
        }
    }
    0
}


//            let l2_hash = murmurhash3::murmurhash3_x86_32(l2_key, 1) >> 16;
//            println!("l2_hash {}", l2_hash);
//            let core_flag = unsafe { lb_filter.offset(l2_hash as isize) };
//            // unsafe {
//            //     let bit_count = core::arch::x86_64::_popcnt64(0xff as i64);
//            //     println!("bit count {}", bit_count);
//            //
//            // }
