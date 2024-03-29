// Note, ALL_CAPS variables will be interpolated at Rust compile-time.

// make ReadableStream async iterable;
// https://bugs.chromium.org/p/chromium/issues/detail?id=929585
ReadableStream.prototype[Symbol.asyncIterator] = async function* () {
  const reader = this.getReader();
  try {
    while (true) {
      const {done, value} = await reader.read();
      if (done) return;
      yield value;
    }
  }
  finally {
    reader.releaseLock();
  }
}


function split_stream(delimiter) {
  let buffer = "";

  return new TransformStream({
    transform(chunk, controller) {
      buffer += chunk;
      const parts = buffer.split(delimiter);
      parts.slice(0, -1).forEach(part => controller.enqueue(part));
      buffer = parts[parts.length - 1];
    },
    flush(controller) {
      if (buffer) controller.enqueue(buffer);
    }
  });
}


async function stream_floats(path, on_float, on_chunk) {
  let res = await fetch(path);

  let remainder = new Uint8Array(0);
  for await (const chunk of res.body) {
    // handle previous remainder, if any
    let offset = (8 - remainder.length) % 8;
    if (0 != offset) {
      let rest = new Uint8Array(chunk.buffer.slice(0, offset));
      on_float(new DataView(new Uint8Array([...remainder, ...rest]).buffer).getFloat64(0));
    }

    const v = new DataView(chunk.buffer);
    while (offset + 8 <= chunk.buffer.byteLength) {
      on_float(v.getFloat64(offset));
      offset += 8;
    }

    remainder = new Uint8Array(chunk.buffer.slice(offset));

    on_chunk();

  }
}


async function stream_text(path, on_line) {
  let res = await fetch(path);
  for await (const line of res.body.pipeThrough(new TextDecoderStream()).pipeThrough(split_stream("\n"))) {
    on_line(line);
  }
}


// on any error, just keep trying to reload the page
async function on_error(e){
  while (true) {
    try {
      await fetch(location.href);
      location.reload();
      break;
    } catch (_) {
      await new Promise(resolve => setTimeout(resolve, 2000));
    }
  }
}

async function main(){
  // TODO: preallocate array with "missing" value (NaN?)
  let data = new Array(N_SERIES);
  for (let i = 0; i < N_SERIES; i++) {
    data[i] = [];
  }

  let uplot = new uPlot(PLOT_OPTS, data, window.splot_chart);

  let n = 0; // number of samples
  let j = 0;
  const on_float = (x) => {

    data[j].push(x);

    // Would be nice to generate this JS on the rust side so there's no comparison/branching around how many series we have.
    j = (j + 1) % N_SERIES;
    if (0 == j) {
      n += 1;
    }
  }

  const on_chunk = () => uplot.setData(data);
  stream_floats("/data", on_float, on_chunk).catch(on_error);
  stream_text("/text", (line) => { window.splot_text.innerText = line + "\n" + window.splot_text.innerText }).catch(on_error);
}



main()
