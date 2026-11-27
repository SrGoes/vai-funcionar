# Máquinas

Todo número de medição (FPS, frametime, memória, censo de shaders) vale para a máquina em que foi medido. Diga sempre em que máquina, perfil de console e rota mediu. Caminhos locais ficam no `CLAUDE.local.md` de cada máquina, fora do git.

## Goes (dono do repositório)

- GPU: NVIDIA GeForce RTX 5060, 8 GB de VRAM, driver 621.40. CPU: Intel i5-13400F (AVX2). RAM: 16 GB (o README recomenda 64 GB; o perfil mais enxuto do projeto mira 32 GB).
- Toolchain: Rust fixado em `rust-toolchain.toml`, VS Build Tools 2026 com C++ e Windows SDK 10.0.26100.
- Cache em 26-27/11/2026: 25.756 shaders traduzidos, 229 sem tradução, sendo 26 programas wave64 sem conversão para wave32, que é o que a NVIDIA executa.

## Bruno (`DESKTOP-BRUNORLAGO`)

- GPU: NVIDIA GeForce RTX 4080, 16 GB de VRAM, driver 621.24. CPU: AMD Ryzen 7 9800X3D. RAM: 32 GB DDR5.
- Sistema: Windows 10 (19045): rode `cargo xtask agility` antes do build.
