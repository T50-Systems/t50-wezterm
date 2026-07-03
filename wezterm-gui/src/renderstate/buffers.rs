pub enum IndexBuffer {
    Glium(GliumIndexBuffer<u32>),
    WebGpu(WebGpuIndexBuffer),
}

impl IndexBuffer {
    pub fn glium(&self) -> &GliumIndexBuffer<u32> {
        match self {
            Self::Glium(g) => g,
            _ => unreachable!(),
        }
    }
    pub fn webgpu(&self) -> &WebGpuIndexBuffer {
        match self {
            Self::WebGpu(g) => g,
            _ => unreachable!(),
        }
    }
}

pub enum VertexBuffer {
    Glium(GliumVertexBuffer<Vertex>),
    WebGpu(WebGpuVertexBuffer),
}

impl VertexBuffer {
    pub fn glium(&self) -> &GliumVertexBuffer<Vertex> {
        match self {
            Self::Glium(g) => g,
            _ => unreachable!(),
        }
    }
    pub fn webgpu(&self) -> &WebGpuVertexBuffer {
        match self {
            Self::WebGpu(g) => g,
            _ => unreachable!(),
        }
    }
    pub fn webgpu_mut(&mut self) -> &mut WebGpuVertexBuffer {
        match self {
            Self::WebGpu(g) => g,
            _ => unreachable!(),
        }
    }
}

enum MappedVertexBuffer {
    Glium(GliumMappedVertexBuffer),
    WebGpu(WebGpuMappedVertexBuffer),
}

impl MappedVertexBuffer {
    fn slice_mut(&mut self, range: std::ops::Range<usize>) -> &mut [Vertex] {
        match self {
            Self::Glium(g) => &mut g.mapping[range],
            Self::WebGpu(g) => {
                let mapping: &mut [Vertex] = bytemuck::cast_slice_mut(&mut g.mapping);
                &mut mapping[range]
            }
        }
    }
}

pub struct MappedQuads<'a> {
    mapping: MappedVertexBuffer,
    next: RefMut<'a, usize>,
    capacity: usize,
}

pub struct WebGpuMappedVertexBuffer {
    mapping: wgpu::BufferViewMut<'static>,
    // Owner mapping, must be dropped after mapping
    _slice: wgpu::BufferSlice<'static>,
}

pub struct WebGpuVertexBuffer {
    buf: wgpu::Buffer,
    num_vertices: usize,
    state: Rc<WebGpuState>,
}

impl std::ops::Deref for WebGpuVertexBuffer {
    type Target = wgpu::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl WebGpuVertexBuffer {
    pub fn new(num_vertices: usize, state: &Rc<WebGpuState>) -> Self {
        Self {
            buf: state.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                size: (num_vertices * std::mem::size_of::<Vertex>()) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: true,
            }),
            num_vertices,
            state: Rc::clone(state),
        }
    }

    pub fn map(&self) -> WebGpuMappedVertexBuffer {
        unsafe {
            let slice = self.buf.slice(..).extend_lifetime();
            let mapping = slice.get_mapped_range_mut();

            WebGpuMappedVertexBuffer {
                mapping,
                _slice: slice,
            }
        }
    }

    pub fn recreate(&mut self) -> wgpu::Buffer {
        let mut new_buf = self.state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: (self.num_vertices * std::mem::size_of::<Vertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: true,
        });
        std::mem::swap(&mut new_buf, &mut self.buf);
        new_buf
    }
}

pub struct WebGpuIndexBuffer {
    buf: wgpu::Buffer,
}

impl std::ops::Deref for WebGpuIndexBuffer {
    type Target = wgpu::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl WebGpuIndexBuffer {
    pub fn new(indices: &[u32], state: &WebGpuState) -> Self {
        Self {
            buf: state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    usage: wgpu::BufferUsages::INDEX,
                    contents: bytemuck::cast_slice(indices),
                }),
        }
    }
}

/// This is a self-referential struct, but since those are not possible
/// to create safely in unstable rust, we transmute the lifetimes away
/// to static and store the owner (RefMut) and the derived Mapping object
/// in this struct
pub struct GliumMappedVertexBuffer {
    mapping: Mapping<'static, [Vertex]>,
    // Drop the owner after the mapping
    _owner: RefMut<'static, VertexBuffer>,
}

impl<'a> QuadAllocator for MappedQuads<'a> {
    fn allocate<'b>(&'b mut self) -> anyhow::Result<QuadImpl<'b>> {
        let idx = *self.next;
        *self.next += 1;
        let idx = if idx >= self.capacity {
            // We don't have enough quads, so we'll keep re-using
            // the first quad until we reach the end of the render
            // pass, at which point we'll detect this condition
            // and re-allocate the quads.
            0
        } else {
            idx
        };

        let idx = idx * VERTICES_PER_CELL;
        let mut quad = Quad {
            vert: self.mapping.slice_mut(idx..idx + VERTICES_PER_CELL),
        };

        quad.set_has_color(false);

        Ok(QuadImpl::Vert(quad))
    }

    fn extend_with(&mut self, vertices: &[Vertex]) {
        let idx = *self.next;
        let len = vertices.len();

        // idx and next are number of quads, so divide by number of vertices
        *self.next += len / VERTICES_PER_CELL;
        // Only copy in if there is enough room.
        // We'll detect the out of space condition at the end of
        // the render pass.
        let idx = idx * VERTICES_PER_CELL;
        let capacity = self.capacity * VERTICES_PER_CELL;
        if idx + len <= capacity {
            self.mapping
                .slice_mut(idx..idx + len)
                .copy_from_slice(vertices);
        }
    }
}
