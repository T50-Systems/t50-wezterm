pub struct TripleVertexBuffer {
    pub index: RefCell<usize>,
    pub bufs: RefCell<[VertexBuffer; 3]>,
    pub indices: IndexBuffer,
    pub capacity: usize,
    pub next_quad: RefCell<usize>,
}

/// A trait to avoid broadly-scoped transmutes; we only want to
/// transmute to extend a lifetime to static, and not to change
/// the underlying type.
/// These ExtendStatic trait impls constrain the transmutes in that way,
/// so that the type checker can still catch issues.
unsafe trait ExtendStatic {
    type T;
    unsafe fn extend_lifetime(self) -> Self::T;
}

unsafe impl<'a, T: 'static> ExtendStatic for Ref<'a, T> {
    type T = Ref<'static, T>;
    unsafe fn extend_lifetime(self) -> Self::T {
        std::mem::transmute(self)
    }
}

unsafe impl<'a, T: 'static> ExtendStatic for RefMut<'a, T> {
    type T = RefMut<'static, T>;
    unsafe fn extend_lifetime(self) -> Self::T {
        std::mem::transmute(self)
    }
}

unsafe impl<'a> ExtendStatic for wgpu::BufferSlice<'a> {
    type T = wgpu::BufferSlice<'static>;
    unsafe fn extend_lifetime(self) -> Self::T {
        std::mem::transmute(self)
    }
}

unsafe impl<'a> ExtendStatic for MappedQuads<'a> {
    type T = MappedQuads<'static>;
    unsafe fn extend_lifetime(self) -> Self::T {
        std::mem::transmute(self)
    }
}

unsafe impl<'a, T: ?Sized + ::window::glium::buffer::Content + 'static> ExtendStatic
    for BufferMutSlice<'a, T>
{
    type T = BufferMutSlice<'static, T>;
    unsafe fn extend_lifetime(self) -> Self::T {
        std::mem::transmute(self)
    }
}

impl TripleVertexBuffer {
    pub fn clear_quad_allocation(&self) {
        *self.next_quad.borrow_mut() = 0;
    }

    pub fn need_more_quads(&self) -> Option<usize> {
        let next = *self.next_quad.borrow();
        if next > self.capacity {
            Some(next)
        } else {
            None
        }
    }

    pub fn vertex_index_count(&self) -> (usize, usize) {
        let num_quads = *self.next_quad.borrow();
        (num_quads * VERTICES_PER_CELL, num_quads * INDICES_PER_CELL)
    }

    pub fn map(&self) -> MappedQuads<'_> {
        let mut bufs = self.current_vb_mut();

        // To map the vertex buffer, we need to hold a mutable reference to
        // the buffer and hold the mapping object alive for the duration
        // of the access.  Rust doesn't allow us to create a struct that
        // holds both of those things, because one references the other
        // and it doesn't permit self-referential structs.
        // We use the very blunt instrument "transmute" to force Rust to
        // treat the lifetimes of both of these things as static, which
        // we can then store in the same struct.
        // This is "safe" because we carry them around together and ensure
        // that the owner is dropped after the derived data.
        let mapping = match &mut *bufs {
            VertexBuffer::Glium(vb) => {
                let buf_slice = unsafe {
                    vb.slice_mut(..)
                        .expect("to map vertex buffer")
                        .extend_lifetime()
                };
                let mapping = buf_slice.map();

                MappedVertexBuffer::Glium(GliumMappedVertexBuffer {
                    _owner: bufs,
                    mapping,
                })
            }
            VertexBuffer::WebGpu(vb) => MappedVertexBuffer::WebGpu(vb.map()),
        };

        MappedQuads {
            mapping,
            next: self.next_quad.borrow_mut(),
            capacity: self.capacity,
        }
    }

    pub fn current_vb_mut(&self) -> RefMut<'static, VertexBuffer> {
        let index = *self.index.borrow();
        let bufs = self.bufs.borrow_mut();
        unsafe { RefMut::map(bufs, |bufs| &mut bufs[index]).extend_lifetime() }
    }

    pub fn next_index(&self) {
        let mut index = self.index.borrow_mut();
        *index += 1;
        if *index >= 3 {
            *index = 0;
        }
    }
}
