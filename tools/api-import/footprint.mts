import { execFileSync } from 'node:child_process'
import { readFileSync, writeFileSync, statSync } from 'node:fs'
import { join } from 'node:path'
const root = process.argv[2]
if (!root) throw Error('Workspace required')
const rows = []
for (const name of ['baseline', 'candidate']) {
  const cwd = join(root, name), started = performance.now()
  execFileSync('cargo', ['+1.97.1', 'build', '--locked', '--release', '-p', 'zixcel-google-cli', '--bin', 'zix-google'], {
    cwd, stdio: 'inherit', timeout: 600000,
    env: { ...process.env, CARGO_TARGET_DIR: join(root, `${name}-cold-target`) },
  })
  const elapsed = Math.round(performance.now() - started)
  const sources = execFileSync('git', ['ls-files', '*.rs'], {cwd, encoding:'utf8'}).trim().split('\n')
  rows.push({ name, compileElapsedMs: elapsed, coldTargetDirectory: true,
    binaryBytes: statSync(join(root, `${name}-cold-target/release/zix-google`)).size,
    lockedPackages: [...readFileSync(join(cwd,'Cargo.lock'),'utf8').matchAll(/^\[\[package\]\]$/gm)].length,
    rustSourceFiles: sources.length, rustSourceBytes: sources.reduce((n,p)=>n+statSync(join(cwd,p)).size,0),
  })
}
writeFileSync(join(root,'footprint.json'), JSON.stringify({schema:'zix-google-footprint/v1',
  baseline:'7ceb3d119d6b1e78405ed1a9950b7e6284c0cda3', rows,
  limits:'Runner-specific timings; separate cold targets share download cache. No size/speed improvement claimed.'},null,2)+'\n')
console.log(JSON.stringify(rows))
