//! Defines the receive and transmit queues that store a ring of DMA descriptors and related information.
//! 
//! Receive and transmit queues are used across all NICs to keep track of incoming and outgoing packets.
//! HW queues used by the NIC only consist of the ring of DMA descriptors. The SW queues defined here hold the ring of DMA descriptors that it shares with the HW,
//! as well as other information such as the buffers received from the queues, the tail register for each queue and the cpu the queue is mapped to.

#![no_std]

extern crate alloc;
extern crate log;
extern crate memory;
extern crate intel_ethernet;
extern crate nic_buffers;
extern crate owning_ref;

use owning_ref::BoxRefMut;
use alloc::{
    vec::Vec,
    collections::VecDeque
};
use memory::MappedPages;
use intel_ethernet::descriptors::{RxDescriptor, TxDescriptor};
use nic_buffers::{ReceiveBuffer, ReceivedFrame};


/// A struct that holds all information for one receive queue.
/// There should be one such object per queue.
pub struct RxQueue<T: RxDescriptor> {
    /// The number of the queue, stored here for our convenience.
    pub id: u8,
    /// Receive descriptors
    pub rx_descs: BoxRefMut<MappedPages, [T]>,
    /// Current receive descriptor index
    pub rx_cur: u16,
    /// The list of rx buffers, in which the index in the vector corresponds to the index in `rx_descs`.
    /// For example, `rx_bufs_in_use[2]` is the receive buffer that will be used when `rx_descs[2]` is the current rx descriptor (rx_cur = 2).
    pub rx_bufs_in_use: Vec<ReceiveBuffer>,
    /// The queue of received Ethernet frames, ready for consumption by a higher layer.
    /// Just like a regular FIFO queue, newly-received frames are pushed onto the back
    /// and frames are popped off of the front.
    /// Each frame is represented by a Vec<ReceiveBuffer>, because a single frame can span multiple receive buffers.
    /// TODO: improve this? probably not the best cleanest way to expose received frames to higher layers   
    pub received_frames: VecDeque<ReceivedFrame>,
    /// The cpu which this queue is mapped to. 
    /// This in itself doesn't guarantee anything, but we use this value when setting the cpu id for interrupts and DCA.
    pub cpu_id: u8,
}



/// A struct that holds all information for a transmit queue. 
/// There should be one such object per queue.
pub struct TxQueue<T: TxDescriptor> {
    /// The number of the queue, stored here for our convenience.
    pub id: u8,
    /// Transmit descriptors 
    pub tx_descs: BoxRefMut<MappedPages, [T]>,
    /// Current transmit descriptor index
    pub tx_cur: u16,
    /// The cpu which this queue is mapped to. 
    /// This in itself doesn't guarantee anything but we use this value when setting the cpu id for interrupts and DCA.
    pub cpu_id : u8,
}
