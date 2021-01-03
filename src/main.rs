#![allow(dead_code)]
use std::fmt::Debug;
use std::marker::PhantomData;
use vec_arena::Arena;

trait Kernel {
    fn work(&mut self);
    fn init(&mut self) {}
    fn deinit(&mut self) {}
}

struct StreamIO {}
struct MessageIO<K> {
    p: PhantomData<K>,
}
struct BlockMeta {}

struct Block<K: Kernel> {
    meta: BlockMeta,
    mio: MessageIO<K>,
    sio: StreamIO,
    kernel: K,
}

trait BlockT {
    fn work(&mut self);
    fn init(&mut self);
    // much more...
}

impl<K: Kernel> BlockT for Block<K> {
    fn work(&mut self) {
        self.kernel.work();
    }
    fn init(&mut self) {
        self.kernel.init();
    }
}

struct Topology {
    blocks: Arena<Box<dyn BlockT>>,
}

struct Flowgraph {
    topology: Topology,
}

impl Flowgraph {
    pub fn start(&mut self) { todo!() }
    pub fn stop(&mut self) { todo!() }
    pub fn run(&mut self) { todo!() }
}

struct VectorSource<T: Debug> {
    items: Vec<T>,
}

impl<T: Debug> Kernel for VectorSource<T> {
    fn work(&mut self) {
       println!("I'm a vector source {:?}.", self.items)
    }
}

fn main() {

    let vector_source_int = Box::new(Block {
       meta: BlockMeta {},
       sio: StreamIO {},
       mio: MessageIO {p: PhantomData},
       kernel: VectorSource {
           items: vec![1, 2, 3],
       },
    });

    let vector_source_float = Box::new(Block {
       meta: BlockMeta {},
       sio: StreamIO {},
       mio: MessageIO {p: PhantomData},
       kernel: VectorSource {
           items: vec![1.0, 2.0, 3.0],
       },
    });

    let mut a : Arena<Box<dyn BlockT>> = Arena::new();
    a.insert(vector_source_int);
    a.insert(vector_source_float);

    let mut f = Flowgraph {
        topology: Topology {
            blocks: a,
        },
    };

    f.topology.blocks[0].work();
    f.topology.blocks[1].work();
}
