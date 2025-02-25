async function update_registers() {
    const response = await fetch('/registers');
    const data = await response.json();

    const reg_names = ["R0", "R1", "R2", "R3", "R4", "R5", "R6", "R7", "R8", "R9", "R10", "R11", "SP", "BF", "LR", "PC"];

    const registersList = document.getElementById('registers-list');
    registersList.innerHTML = '';

    for (const [i, value] of reg_names.entries()) {
        registersList.innerHTML += `
        <div class="col-6 col-lg-3 mb-2">
            <div class="border rounded p-2">
                <small>${value}</small>
                <div class="font-monospace">${data[i]}</div>
            </div>
        </div>`
    }
}

async function update_pipeline() {
    const response = await fetch('/processor/pipeline');
    const data = await response.json();

    const status = await fetch('/processor/pipeline/status');
    const status_data = await status.json();

    const table = document.getElementById('pipeline-table');
    const row = table.getElementsByTagName("tr")[1];
    
    for (const [i, element] of data.entries()) {
        let td = row.getElementsByTagName("td")[i];
        if (element != null) {
            td.innerHTML = `
                Stage Status: ${status_data[i]} <br>
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

async function update_cycles() {
    const response = await fetch('/processor/cycles');
    const data = await response.json();

    document.getElementById('cycles-count').innerHTML = `Cycles: ${data}`;
}

async function step() {
    await fetch('/step');
    await update_registers();
    await update_pipeline();
    await update_cycles();
}

async function run() {
    await fetch('/run');
    await update_registers();
    await update_pipeline();
    await update_cycles();
}

async function reset() {
    await fetch('/reset');
    await update_registers();
    await update_pipeline();
    await update_cycles();
}

async function main() {
    await update_registers();
    await update_pipeline();

    document.getElementById('step-button').onclick = step;
    document.getElementById('run-button').onclick = run;
    document.getElementById('reset-button').onclick = reset;

    setInterval(async () => {
        await update_registers();
        await update_pipeline();
        await update_cycles();
    }, 1000);
}

main();