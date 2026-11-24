use cubecl::prelude::*;
use cubecl::wgpu::{AutoCompiler, WgpuDevice, WgpuRuntime};
use cubecl::bytes::Bytes;
use cubecl::frontend::BufferArg;
use cubecl_runtime::runtime::Runtime;

#[cube(launch)]
fn double_values<F: Float>(input: &[F], output: &mut [F]) {
    if ABSOLUTE_POS < input.len() {
        output[ABSOLUTE_POS] = input[ABSOLUTE_POS] * F::new(2.0f32);
    }
}

fn main() {
    let device = WgpuDevice::default();
    let client = WgpuRuntime::<AutoCompiler>::client(&device);

    let input_data : [f32; 8] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let num_elements = input_data.len();

    let output_handle = client.create(Bytes::from_elems(input_data.to_vec()));

    for tick in 0..=100 {
        double_values::launch::<f32>(
            &client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_1d(num_elements as u32),
            unsafe { BufferArg::from_raw_parts(output_handle.clone(), num_elements) },
            unsafe { BufferArg::from_raw_parts(output_handle.clone(), num_elements) },
        );

        let output_bytes = client.read_one(output_handle.clone()).unwrap();
        let output_data = f32::from_bytes(&output_bytes);

        println!("Tick {tick}: {output_data:?}");
    }
}