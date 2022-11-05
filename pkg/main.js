function load_png_data(evt) {
  let file = evt.target.files[0];

  const reader = new FileReader();

  reader.addEventListener("load", (event) => {
    let buffer = reader.result;
    let typedArray = new Uint8Array(buffer);
    window.inputted_bytes = typedArray;
    let textarea = document.getElementById("main-input-box");
    textarea.innerText = "[" + typedArray.toString() + "]"
  })

  reader.readAsArrayBuffer(file);
}

function get_bytes(id) {
  let token_bits = document.getElementsByClassName("token-bits");
  let valid = [];
  for (let token of token_bits) {
    if (token.getAttribute("data-token") == id) {
      valid.push(token);
    }
  }
  return valid;
}

function token_hover_over(id) {
  let tokens = get_bytes(id);
  for (let token of tokens) {
    token.classList.add("highlighted-token")
  }
  moveByteToView(tokens[0]);
}

function token_hover_out(id) {
  let tokens = get_bytes(id);
  for (let token of tokens) {
    token.classList.remove("highlighted-token")
  }
}

function decode_data(evt) {
  let textarea = document.getElementById("main-input-box");
  let text_data = textarea.value;
  
  try {
    let array = JSON.parse(text_data);
    let typed_array = new Uint8Array(array);
    window.inputted_bytes = typed_array;
    data = typed_array;
  } catch (err) {
    alert("Can't decode data" + err)
    return;
  }
  if (data[0] == 137 && data[1] == 80 && data[2] == 78 && data[3] == 71) {
    // PNG
    window.call_wasm_decode_png(data)
  } else {
    window.call_wasm_decode_zlib(data)
  }
}

function construct_token_row(token, i) {
  let tr = document.createElement("tr");
  let td = document.createElement("td");

  let divParent = document.createElement("div");
  divParent.addEventListener("mouseover", (evt) => {
    token_hover_over(i)
  });
  divParent.addEventListener("mouseout", (evt) => {
    token_hover_out(i)
  });
  divParent.classList.add("token-row");
  divParent.classList.add("token-row-nest-" + token.nest_level);

  let tokenTypeDiv = document.createElement("div");
  tokenTypeDiv.classList.add("token-type-parent");

  let tokenTypeText = document.createElement("div");
  tokenTypeText.classList.add("token-type");
  tokenTypeText.classList.add("tooltip");
  tokenTypeText.innerText = token.token_type;

  let tooltipText = document.createElement("span");
  tooltipText.classList.add("tooltiptext");
  tooltipText.innerText = token.description;

  tokenTypeText.appendChild(tooltipText);

  let tokenDataDiv = document.createElement("div");
  tokenDataDiv.classList.add("token-data-parent");

  let tokenDataText = document.createElement("span");
  tokenDataText.classList.add("token-data");
  tokenDataText.innerText = token.data;

  tokenTypeDiv.appendChild(tokenTypeText);
  tokenDataDiv.appendChild(tokenDataText);

  divParent.appendChild(tokenTypeDiv);
  divParent.appendChild(tokenDataDiv);

  td.appendChild(divParent);
  tr.appendChild(td);

  return tr

    /*
  `<tr>
      <td>
        <div onmouseover="token_hover_over(${i})" onmouseout="token_hover_out(${i})" class="token-row token-row-nest-${token.nest_level}">
          <div class="token-type-parent">
            <text class="token-type">${token.token_type}</text>
          </div>
          <div class="token-data-parent">
            <text class="token-data">${token.data}</text>
          </div>
        </div>
      </td>
    </tr>`; 
    */
}

function decoded_data_callback(decoded_tokens) {
  document.getElementById("tables-body").style.visibility = "visible";

  let token_table = document.createElement("tbody");
  let bytes_table = document.createElement("tbody");

  // empty table with headers
  bytes_table.innerHTML = `<tr style="width: 100%;">
      <th>0</th>
      <th>1</th>
      <th>2</th>
      <th>3</th>
      <th>4</th>
      <th>5</th>
      <th>6</th>
      <th>7</th>
  </tr>
  <tr>

  </tr>`;

  var remaining_in_byte = 0;
  var byte_col = 0;
  var current_byte_span = null;

  for (const [index, token] of decoded_tokens.entries()) {
    let html = construct_token_row(token, index);
    token_table.appendChild(html);

    if (token.using_bytes) {
      var bits = bytes_to_bits(token.bits);
    } else {
      var bits = token.bits;
    }

    var current_token;
    if (current_byte_span != null) {
      current_token = add_token_span(index, current_byte_span);
    }

    for (let bit of bits) {
      if (remaining_in_byte == 0) {
        // end of byte
        current_byte_span = add_new_byte_span(bytes_table);
        current_token = add_token_span(index, current_byte_span);
        remaining_in_byte = 8;
        byte_col += 1;

        if (byte_col == 8) {
          bytes_table.appendChild(document.createElement("tr"));
          byte_col = 0;
        }
      }

      // bytes should be read the other way, as they are meant to be interpreted from the msb in the stream, but read from lsb
      if (token.using_bytes) {
        current_token.innerText = current_token.innerText + bit;
      } else {
        current_token.innerText = bit + current_token.innerText;
      }
      remaining_in_byte -= 1;
    }
  }
  let token_table_element = document.getElementById("token-table");
  token_table_element.textContent = "";
  token_table_element.appendChild(token_table);

  let byte_table_element = document.getElementById("bytes-table");
  byte_table_element.textContent = "";
  byte_table_element.appendChild(bytes_table);
}
window.decoded_callback = decoded_data_callback;

function add_new_byte_span(bytes_table) {
  let row = bytes_table.lastChild;
  
  let tableData = document.createElement("td");
  let span = document.createElement("span");
  span.classList.add("byte");
  tableData.appendChild(span);
  row.appendChild(tableData);

  return span
}

function add_token_span(id, current_byte_span) {
  let span = document.createElement("span");
  span.classList.add("token-bits");
  span.setAttribute("data-token", id);

  current_byte_span.prepend(span);

  return span
}

function one_byte_to_bits(byte) {
  return [byte&1, (byte&2)>>1, (byte&4)>>2, (byte&8)>>3, (byte&16)>>4, (byte&32)>>5, (byte&64)>>6, (byte&128)>>7].reverse()
}

function bytes_to_bits(bytes) {
  let all_bits = [];
  for (let byte of bytes) {
    let bits = one_byte_to_bits(byte);
    all_bits = all_bits.concat(bits);
  }
  return all_bits
}

function moveByteToView(byte) {
  let bytesTable = document.getElementById("bytes-table")

  let byteTop = byte.getBoundingClientRect().top;
  let middleScreen = window.innerHeight / 4;
  let shiftY = -(byteTop - middleScreen);

  bytesTable.style.top = Math.max(0, bytesTable.offsetTop + shiftY) + "px";
}
