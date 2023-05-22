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

async function main(){
  // TODO: preallocate array with "missing" value (NaN?)
  let data = new Array(N_SERIES);
  for (let i = 0; i < N_SERIES; i++) {
    data[i] = [];
  }

  let uplot = new uPlot(PLOT_OPTS, data, document.body);

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
  stream_floats("/data", on_float, on_chunk);
}



main()
