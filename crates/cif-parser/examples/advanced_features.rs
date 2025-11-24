// Demonstrate save frames and multiple data blocks
use cif_parser::Document;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let cif_content = r#"
data_block1
_item1 value1

save_frame1
_frame_item 'frame value'
loop_
_frame_loop.id
_frame_loop.value
1 'first'
2 'second'
save_

data_block2
_item2 value2
"#;

    let doc = Document::parse(cif_content)?;
    println!("Number of data blocks: {}", doc.blocks.len());

    // Access multiple blocks
    for block in &doc.blocks {
        println!("\nData block: {}", block.name);
        println!("  Items: {}", block.items.len());
        println!("  Loops: {}", block.loops.len());
        println!("  Frames: {}", block.frames.len());

        // Access save frames
        for frame in &block.frames {
            println!("  Save frame: {}", frame.name);
            for (tag, value) in &frame.items {
                println!("    {}: {:?}", tag, value);
            }
        }
    }

    Ok(())
}
