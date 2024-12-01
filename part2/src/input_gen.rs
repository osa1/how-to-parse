use oorandom::Rand64;

const RAND_SEED: u128 = 123;
const MAX_STR_SIZE: u64 = 10; // bytes

/// Generates JSON input, for testing and benchmarking.
pub fn gen_input(size: usize) -> String {
    let mut rng = Rand64::new(RAND_SEED);

    // Allocate larger than `size`: atoms (numbers, strings, keywords) can be larger than the
    // remaining space, objects allocate key + value regardless of the remaining size, containers
    // don't take commas and closing characters into account... so we generate more.
    let mut output = String::with_capacity(size + 1000);

    let mut container_stack: Vec<Container> = vec![];

    // Always start with an array or object, to be able to generate enough
    // elements to fill up the space.
    if rand_bool(&mut rng) {
        // Array
        output.push('[');
        container_stack.push(Container::Array);
    } else {
        output.push('{');
        container_stack.push(Container::Object);
    }

    while output.len() < size {
        // If in an object, generate a key first.
        if matches!(container_stack.last(), Some(Container::Object)) {
            gen_str(&mut rng, &mut output);
            output.push(':');
        }

        match rng.rand_range(0..6) {
            // Int
            0 => {
                let int = rng.rand_u64();
                let int_str = int.to_string();
                output.push_str(&int_str);
            }

            // String
            1 => {
                gen_str(&mut rng, &mut output);
            }

            // Bool
            2 => {
                if rand_bool(&mut rng) {
                    output.push_str("true");
                } else {
                    output.push_str("false");
                }
            }

            // Null
            3 => {
                output.push_str("null");
            }

            // Array
            4 => {
                output.push('[');
                container_stack.push(Container::Array);
                continue;
            }

            // Object
            5 => {
                output.push('{');
                container_stack.push(Container::Object);
                continue;
            }

            _ => unreachable!(),
        }

        if container_stack.len() > 1 && rand_bool(&mut rng) {
            match container_stack.pop().unwrap() {
                Container::Array => output.push(']'),
                Container::Object => output.push('}'),
            }
        }

        if output.len() > size {
            break;
        }

        output.push(',');
    }

    // Terminate containers.
    while let Some(container) = container_stack.pop() {
        match container {
            Container::Array => output.push(']'),
            Container::Object => output.push('}'),
        }
    }

    output
}

#[derive(Debug, Clone, Copy)]
enum Container {
    Array,
    Object,
}

fn rand_bool(rng: &mut Rand64) -> bool {
    rng.rand_range(0..2) == 1
}

fn gen_str(rng: &mut Rand64, output: &mut String) {
    let str_size = rng.rand_range(0..MAX_STR_SIZE + 1);
    output.push('"');
    for _ in 0..str_size {
        // Only generate ('a'..='z') to make it easy to visually inspect the output.
        let char = rng.rand_range(u64::from(b'a')..u64::from(b'z') + 1) as u8 as char;
        output.push(char);
    }
    output.push('"');
}
