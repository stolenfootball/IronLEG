async function update_registers(reg_contents, reg_status) {
    const reg_names = ["R0", "R1", "R2", "R3", "R4", "R5", "R6", "R7", "R8", "R9", "R10", "R11", "SP", "BF", "LR", "PC"];

    const registersList = document.getElementById('registers-list');
    registersList.innerHTML = '';

    for (const [i, value] of reg_names.entries()) {
        registersList.innerHTML += `
        <div class="col-6 col-lg-3 mb-2">
            <div class="border rounded p-2 ${reg_status[i] ? 'bg-warning' : ''}">
                <small>${value}</small>
                <div class="font-monospace">${reg_contents[i]}</div>
            </div>
        </div>`
    }
}

async function update_pipeline(pipe_contents, pipe_status) {
    const table = document.getElementById('pipeline-table');
    const row = table.getElementsByTagName("tr")[1];

    for (const [i, element] of pipe_contents.entries()) {
        let td = row.getElementsByTagName("td")[i];
        if (element != null) {
            td.innerHTML = `
                Stage Status: ${pipe_status[i]} <br>
                Raw Instruction: ${element.instr_raw} <br>
            `;
            if (i > 0) {
                td.innerHTML += `
                    Instruction Type: ${Object.keys(element.instr_type)} ${element.instr_type[Object.keys(element.instr_type)]} <br>
                    Address Mode: ${element.addr_mode} <br>
                    Register 1: ${element.reg_1} <br>
                    Register 2: ${element.reg_2} <br>
                    Immediate: ${element.imm} <br>
                    Result: ${element.meta.result} <br>
                    Squashed: ${element.meta.squashed} <br>
                    Writeback: ${element.meta.writeback} <br></br>
                `;
            }

        } else {
            td.innerHTML = 'EMPTY';
        }
    };
}

async function update_memory(data) {
    const tableSpace = document.getElementById("memory-table-space");
    tableSpace.innerHTML = '';

    for (let i = 0; i < data[0].length; i++) {
        const tbl = document.createElement('table');
        tbl.className = "table table-sm table-bordered table-hover";

        const tr = tbl.createTHead().insertRow();
        tr.insertCell().innerHTML = "<b>Address</b>";

        for (let k = 0; k < data[0][0].length; k++) {
            tr.insertCell().innerHTML = `<b>+${k * 4}</b>`;
        }

        for (let x = 0; x < data.length; x++) {
            const tr = tbl.insertRow();
            tr.insertCell().innerHTML = `+${x * 64}`;
            for (let j = 0; j < data[0][0].length; j++) {
                tr.insertCell().innerHTML = data[x][data[0].length - i - 1][j];
            }
        }
        tableSpace.appendChild(tbl);
    }
}

async function update_cycles() {
    const response = await fetch('/cycles');
    const data = await response.json();

    document.getElementById('cycles-count').innerHTML = `Cycles: ${data}`;
}

async function flash() {
    let content = document.getElementById('leg-code').value;
    await fetch('/flash', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({program: content})
    });
    await refresh_ui();
}

async function refresh_ui() {
    const line = document.getElementById('memory-address-box').value;
    console.log(line ? line : '0');

    const response = await fetch('/refresh/' + (line ? line : '0'));
    const data = await response.json();

    document.getElementById('cycles-count').innerHTML = `Cycles: ${data.num_cycles}`;

    await update_registers(data.register_values, data.register_status);
    await update_pipeline(data.pipeline_values, data.pipeline_status);
    await update_memory(data.memory_contents);
}

async function step() {
    await fetch('/step');
    await refresh_ui();
}

async function run() {
    await fetch('/run');
    await refresh_ui();
}

async function reset() {
    await fetch('/reset');
    await refresh_ui();
}

async function main() {
    await refresh_ui();

    document.getElementById('step-button').onclick = step;
    document.getElementById('run-button').onclick = run;
    document.getElementById('reset-button').onclick = reset;
    document.getElementById('flash-button').onclick = flash;
    document.getElementById('memory-button').onclick = update_memory;

//     setInterval(async () => {
//         await update_cycles();
//     }, 100);
}

main();