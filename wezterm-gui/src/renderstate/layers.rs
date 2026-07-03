pub struct RenderLayer {
    pub vb: RefCell<[TripleVertexBuffer; 3]>,
    context: RenderContext,
    zindex: i8,
}

impl RenderLayer {
    pub fn new(context: &RenderContext, num_quads: usize, zindex: i8) -> anyhow::Result<Self> {
        let vb = [
            Self::compute_vertices(context, 32)?,
            Self::compute_vertices(context, num_quads)?,
            Self::compute_vertices(context, 32)?,
        ];

        Ok(Self {
            context: context.clone(),
            vb: RefCell::new(vb),
            zindex,
        })
    }

    pub fn clear_quad_allocation(&self) {
        for vb in self.vb.borrow().iter() {
            vb.clear_quad_allocation();
        }
    }

    pub fn quad_allocator(&self) -> TripleLayerQuadAllocator<'_> {
        // We're creating a self-referential struct here to manage the lifetimes
        // of these related items.  The transmutes are safe because we're only
        // transmuting the lifetimes (not the types), and we're keeping hold
        // of the owner in the returned struct.
        unsafe {
            let vbs = self.vb.borrow().extend_lifetime();
            let layer0 = vbs[0].map().extend_lifetime();
            let layer1 = vbs[1].map().extend_lifetime();
            let layer2 = vbs[2].map().extend_lifetime();
            TripleLayerQuadAllocator::Gpu(BorrowedLayers {
                layers: [layer0, layer1, layer2],
                _owner: vbs,
            })
        }
    }

    pub fn need_more_quads(&self, vb_idx: usize) -> Option<usize> {
        self.vb.borrow()[vb_idx].need_more_quads()
    }

    pub fn reallocate_quads(&self, idx: usize, num_quads: usize) -> anyhow::Result<()> {
        let vb = Self::compute_vertices(&self.context, num_quads)?;
        self.vb.borrow_mut()[idx] = vb;
        Ok(())
    }

    /// Compute a vertex buffer to hold the quads that comprise the visible
    /// portion of the screen.   We recreate this when the screen is resized.
    /// The idea is that we want to minimize any heavy lifting and computation
    /// and instead just poke some attributes into the offset that corresponds
    /// to a changed cell when we need to repaint the screen, and then just
    /// let the GPU figure out the rest.
    fn compute_vertices(
        context: &RenderContext,
        num_quads: usize,
    ) -> anyhow::Result<TripleVertexBuffer> {
        let verts = context.allocate_vertex_buffer_initializer(num_quads);
        log::trace!(
            "compute_vertices num_quads={}, allocated {} bytes",
            num_quads,
            verts.len() * std::mem::size_of::<Vertex>()
        );
        let mut indices = vec![];
        indices.reserve(num_quads * INDICES_PER_CELL);

        for q in 0..num_quads {
            let idx = (q * VERTICES_PER_CELL) as u32;

            // Emit two triangles to form the glyph quad
            indices.push(idx + V_TOP_LEFT as u32);
            indices.push(idx + V_TOP_RIGHT as u32);
            indices.push(idx + V_BOT_LEFT as u32);

            indices.push(idx + V_TOP_RIGHT as u32);
            indices.push(idx + V_BOT_LEFT as u32);
            indices.push(idx + V_BOT_RIGHT as u32);
        }

        let buffer = TripleVertexBuffer {
            index: RefCell::new(0),
            bufs: RefCell::new([
                context.allocate_vertex_buffer(num_quads, &verts)?,
                context.allocate_vertex_buffer(num_quads, &verts)?,
                context.allocate_vertex_buffer(num_quads, &verts)?,
            ]),
            capacity: num_quads,
            indices: context.allocate_index_buffer(&indices)?,
            next_quad: RefCell::new(0),
        };

        Ok(buffer)
    }
}

pub struct BorrowedLayers {
    pub layers: [MappedQuads<'static>; 3],

    // layers references _owner, so it must be dropped after layers.
    _owner: Ref<'static, [TripleVertexBuffer; 3]>,
}

impl TripleLayerQuadAllocatorTrait for BorrowedLayers {
    fn allocate(&mut self, layer_num: usize) -> anyhow::Result<QuadImpl<'_>> {
        self.layers[layer_num].allocate()
    }

    fn extend_with(&mut self, layer_num: usize, vertices: &[Vertex]) {
        self.layers[layer_num].extend_with(vertices)
    }
}
