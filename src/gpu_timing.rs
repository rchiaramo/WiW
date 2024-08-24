pub struct Queries {
    pub set: wgpu::QuerySet,
    resolve_buffer: wgpu::Buffer,
    destination_buffer: wgpu::Buffer,
    num_queries: u64,
    pub next_unused_query: u32,
}

pub struct QueryResults {
    encoder_timestamps: [u64; 2],
    render_start_end_timestamps: [u64; 2],
    compute_start_end_timestamps: [u64; 2],
}

impl QueryResults {
    // Queries:
    // * encoder timestamp start
    // * encoder timestamp end
    // * compute start
    // * compute end
    // * render start
    // * render end
    pub(crate) const NUM_QUERIES: u64 = 6;

    #[allow(clippy::redundant_closure)] // False positive
    pub fn from_raw_results(timestamps: Vec<u64>) -> Self {
        assert_eq!(timestamps.len(), Self::NUM_QUERIES as usize);

        let mut next_slot = 0;
        let mut get_next_slot = || {
            let slot = timestamps[next_slot];
            next_slot += 1;
            slot
        };

        let mut encoder_timestamps = [0, 0];
        encoder_timestamps[0] = get_next_slot();
        let compute_start_end_timestamps = [get_next_slot(), get_next_slot()];
        let render_start_end_timestamps = [get_next_slot(), get_next_slot()];
        encoder_timestamps[1] = get_next_slot();

        QueryResults {
            encoder_timestamps,
            render_start_end_timestamps,
            compute_start_end_timestamps,
        }
    }

    #[cfg_attr(test, allow(unused))]
    pub fn print(&self, queue: &wgpu::Queue) {
        let period = queue.get_timestamp_period();
        let elapsed_ms = |start, end: u64| { end.wrapping_sub(start) as f64 * period as f64 / (1000000.0) };

        println!(
            "Elapsed time before compute until after render: {:.2} ms",
            elapsed_ms(self.encoder_timestamps[0], self.encoder_timestamps[1]) as f32,
        );
        println!(
            "Elapsed time compute pass: {:.2} ms",
            elapsed_ms(
                self.compute_start_end_timestamps[0],
                self.compute_start_end_timestamps[1]
            ) as f32
        );
        println!(
            "Elapsed time render pass: {:.2} ms",
            elapsed_ms(
                self.render_start_end_timestamps[0],
                self.render_start_end_timestamps[1]
            ) as f32
        );
    }
}

impl Queries {
    pub(crate) fn new(device: &wgpu::Device, num_queries: u64) -> Self {
        Queries {
            set: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("Timestamp query set"),
                count: num_queries as _,
                ty: wgpu::QueryType::Timestamp,
            }),
            resolve_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("query resolve buffer"),
                size: size_of::<u64>() as u64 * num_queries,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::QUERY_RESOLVE,
                mapped_at_creation: false,
            }),
            destination_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("query dest buffer"),
                size: size_of::<u64>() as u64 * num_queries,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
            num_queries,
            next_unused_query: 0,
        }
    }

    pub fn resolve(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.resolve_query_set(
            &self.set,
            // TODO(https://github.com/gfx-rs/wgpu/issues/3993): Musn't be larger than the number valid queries in the set.
            0..self.next_unused_query,
            &self.resolve_buffer,
            0,
        );
        encoder.copy_buffer_to_buffer(
            &self.resolve_buffer,
            0,
            &self.destination_buffer,
            0,
            self.resolve_buffer.size(),
        );
    }

    pub fn wait_for_results(&self, device: &wgpu::Device) -> Vec<u64> {
        self.destination_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, |_| ());
        device.poll(wgpu::Maintain::wait()).panic_on_timeout();

        let timestamps = {
            let timestamp_view = self
                .destination_buffer
                .slice(..(std::mem::size_of::<u64>() as wgpu::BufferAddress * self.num_queries))
                .get_mapped_range();
            bytemuck::cast_slice(&timestamp_view).to_vec()
        };

        self.destination_buffer.unmap();

        timestamps
    }
}