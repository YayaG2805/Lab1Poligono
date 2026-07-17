# Lab 1 - Relleno de poligonos

Programa en Rust que rasteriza poligonos con el algoritmo scanline y regla par-impar. Genera `out.png` y `out.bmp`, y tambien puede mostrar el mismo framebuffer en una ventana con raylib-rs.

![Resultado generado](out.png)

## Objetivo

Dibujar y rellenar los poligonos 1, 2, 3 y 4 usando coordenadas absolutas en un lienzo de 800 x 450 pixeles. El poligono 5 se usa como agujero interno del poligono 4, por lo que su interior conserva el color de fondo.

## Tecnologias

- Rust 2021
- Cargo
- raylib-rs para la ventana grafica
- Exportadores PNG y BMP implementados sobre el framebuffer propio

## Ejecucion

```bash
cargo run
```

Genera `out.png`, genera `out.bmp`, abre una ventana con raylib y mantiene el resultado visible hasta cerrar la ventana o presionar Escape.

```bash
cargo run -- --export-only
```

Genera `out.png` y `out.bmp` sin abrir ventana. Este modo sirve para ambientes sin interfaz grafica.

## Pruebas

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -- --export-only
```

## Algoritmo

El relleno usa scanline polygon fill. Para cada linea horizontal se evalua `y + 0.5`, se calculan las intersecciones con todas las aristas no horizontales, se ordenan por X y se rellenan los tramos por parejas: 0-1, 2-3, 4-5, etc.

La regla par-impar permite procesar varios contornos juntos. En el poligono 4 se pasan dos contornos: el exterior y el poligono 5. Al cruzar el borde del agujero cambia la paridad, asi que esa zona queda sin rellenar automaticamente.

Las aristas horizontales se ignoran durante el calculo de intersecciones porque no cruzan una scanline. Los extremos verticales se manejan con un intervalo semiabierto para no contar dos veces un mismo vertice.

## Estructura

```text
.
|-- Cargo.toml
|-- Cargo.lock
|-- README.md
|-- .gitignore
|-- out.png          # generado localmente
|-- out.bmp          # generado localmente
`-- src
    |-- lib.rs
    |-- main.rs
    `-- rasterizer.rs
```

## Regenerar salidas

Ejecuta:

```bash
cargo run -- --export-only
```

Esto sobrescribe `out.png` y `out.bmp` desde el mismo framebuffer usado por la ventana. Esos archivos quedan ignorados por Git para no subir salidas generadas, pero el README referencia `out.png` para que se vea localmente despues de regenerarla.

## Archivos no versionados

No se debe subir `target/`, `build/`, ejecutables, librerias compiladas, archivos objeto, salidas generadas ni configuraciones locales de IDE. `.gitignore` ya cubre esos casos. Si algun archivo compilado fue agregado al indice por accidente, retirarlo con:

```bash
git rm -r --cached target build
```

## Lista de verificacion

- [x] Repositorio organizado.
- [x] `out.bmp` generado localmente.
- [x] `out.png` generado localmente y referenciado en el README.
- [x] Sin `build/`, `target/` ni binarios versionados.
- [x] Poligono 1 relleno y con contorno.
- [x] Poligono 2 relleno y con contorno.
- [x] Poligono 3 relleno y con contorno.
- [x] Poligono 4 relleno, concavidades correctas y contorno visible.
- [x] Poligono 5 usado como agujero, sin relleno interior y con contorno visible.
