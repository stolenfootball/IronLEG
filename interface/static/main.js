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

async function step() {
    await fetch('/step');
    await update_registers();
}

async function main() {
    await update_registers();

    document.getElementById('step-button').onclick = step;
}

main();