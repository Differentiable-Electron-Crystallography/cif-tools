import init, { parse } from '@cif-tools/parser/web';

const inputEl = document.getElementById('input');
const outputEl = document.getElementById('output');
const statsEl = document.getElementById('stats');
const parseBtn = document.getElementById('parse-btn');

// Initialize WASM module
await init();

parseBtn.addEventListener('click', () => {
  const cifContent = inputEl.value;

  if (!cifContent.trim()) {
    outputEl.innerHTML = '<span class="error">Please enter some CIF content</span>';
    statsEl.innerHTML = '';
    return;
  }

  try {
    const startTime = performance.now();
    const doc = parse(cifContent);
    const parseTime = (performance.now() - startTime).toFixed(2);

    // Display stats
    const stats = [
      `Blocks: ${doc.blockCount}`,
      `CIF Version: ${doc.isCif2() ? '2.0' : '1.x'}`,
      `Parse time: ${parseTime}ms`,
    ];
    statsEl.innerHTML = stats.map((s) => `<span class="stat">${s}</span>`).join('');

    // Build output
    const output = [];

    for (let i = 0; i < doc.blockCount; i++) {
      const block = doc.get_block(i);
      output.push(`=== Data Block: ${block.name} ===`);
      output.push(`Items: ${block.numItems}`);
      output.push(`Loops: ${block.numLoops}`);
      output.push(`Save Frames: ${block.numFrames}`);
      output.push('');

      // Show some items
      const itemNames = block.item_names;
      if (itemNames.length > 0) {
        output.push('Items:');
        for (const name of itemNames.slice(0, 10)) {
          const value = block.get_item(name);
          let displayValue = '';

          if (value.is_unknown()) {
            displayValue = '? (unknown)';
          } else if (value.is_not_applicable()) {
            displayValue = '. (not applicable)';
          } else if (value.is_numeric()) {
            displayValue = value.numeric_value;
          } else if (value.is_numeric_with_uncertainty()) {
            displayValue = `${value.numeric_value} (${value.uncertainty_value})`;
          } else if (value.is_list()) {
            displayValue = `[list with ${value.list_value.length} items]`;
          } else if (value.is_table()) {
            displayValue = `{table with ${value.table_value.size} entries}`;
          } else {
            displayValue = value.text_value || '(empty)';
          }

          output.push(`  ${name} = ${displayValue}`);
        }
        if (itemNames.length > 10) {
          output.push(`  ... and ${itemNames.length - 10} more items`);
        }
        output.push('');
      }

      // Show loops
      if (block.numLoops > 0) {
        output.push('Loops:');
        for (let j = 0; j < block.numLoops; j++) {
          const loop = block.get_loop(j);
          output.push(`  Loop ${j + 1}: ${loop.numRows} rows, ${loop.numColumns} columns`);
          output.push(`    Tags: ${loop.tags.join(', ')}`);
        }
        output.push('');
      }
    }

    outputEl.textContent = output.join('\n');
  } catch (error) {
    outputEl.innerHTML = `<span class="error">Parse error: ${error.message || error}</span>`;
    statsEl.innerHTML = '';
  }
});
