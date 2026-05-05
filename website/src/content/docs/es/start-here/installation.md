---
title: Instalación
---

Oxide es una librería asíncrona diseñada para que crear un launcher de Minecraft no sea un dolor de cabeza. Nos encargamos de las librerías, los assets y la JVM por ti. Diseñada principalmente para su uso en Tauri.

Añádela de esta manera tan fácil

```bash
cargo add oxide_mc
```
Esta herramienta está enfocada para launchers de series, modpacks individuales, etc., no para launchers completos como Modrinth o GDLauncher, igualmente se podría hacer, pero no es el enfoque.

# Guía de Uso

## Paso 1: Inicialización

Oxide MC es más que un simple objeto; actúa como el núcleo que gestiona y guarda la sesión

Para que el nombre de la sesión sea variable se recomienda directamente guardarlo en un `useState` de React e irlo proporcionando a través de Tauri o similares

```rust
let launcher = OxideLauncher::new("Steve");
```

En este momento la librería nunca ha tocado el disco, solo ha guardado las configuraciones necesarias.

## Paso 2: El flujo de instalación

Este es el punto donde la mayoría se pierde. Divídelo visualmente:

- Check: **¿Está instalado?** (check_game)

```rust
async fn check_game_installed() -> anyhow::Result<()> {
    let launcher = OxideLauncher::new("TestUser");
    let result = launcher.check_game("1.20.1", "vanilla").await?;
    println!("{:#?}", result);
    Ok(())
}
```

- Download: Si no, **descargar todo** (full_install).

Esta función descarga todos los assets y librerías de Minecraft, (Y de Fabric si se ha seleccionado).

```rust
async fn test_install() -> anyhow::Result<()> {
    let launcher = OxideLauncher::new("TestUser");

    launcher.full_install(None, "1.20.1", oxide_mc::state::models::ModLoader::Fabric, false).await?;
    Ok(())
}
```

- Runtime: **Instalar java** (java_download).

Oxide MC maneja Java de forma portable para hacerlo mucho más fácil y más automatico.

Si no se descarga la versión de Java indicada para la versión que se va a instalar puede fallar terriblemente.


| Versión de Minecraft | Java Recomendado | Comando Oxide       |
|:---------------------|:----------------:|:--------------------|
| **1.20.5 — 1.21.x**  |     Java 21      | `java_download(21)` |
| **1.18 — 1.20.4**    |     Java 17      | `java_download(17)` |
| **1.17**             |     Java 16      | `java_download(16)` |
| **1.12.2 — 1.16.5**  |   Java 8 / 11    | `java_download(8)`  |
| **1.7.10 — 1.12.1**  |      Java 8      | `java_download(8)`  |
| **Versiones Legacy** |      Java 8      | `java_download(8)`  |

```rust
async fn java_download() -> anyhow::Result<()> {
    let mut launcher = OxideLauncher::new("TestUser");

    launcher.java_download(17).await?;
    Ok(())
}

```

- Recomendaciones

Intenta hacerlo todo en una función (`intall_minecraft`) así se unifica todo mucho más y no tienes que estar todo el rato dando la variable `name` y sobreescribiendo los valores de configuración.

## Paso 3: Iniciar el juego

Para hacerlo de una manera **más facil** la librería detecta automaticamente que versión has instalado antes por el comando `test_install()` para hacer un comando unificado, estetico y sencillo.

```rust
async fn run() -> anyhow::Result<()> {
    let launcher = OxideLauncher::new("TestUser");

    launcher.start("8G").await?;  // 8G es el argumento de cuanta RAM tiene que utilizar el juego
    Ok(())
}
```

## Comandos de Check

```rust
async fn check_java_version_test() -> anyhow::Result<()> {
    let launcher = OxideLauncher::new("TestUser");
    launcher.check_java().await?;
    Ok(())
}

async fn check_game_installed() -> anyhow::Result<()> {
    let launcher = OxideLauncher::new("TestUser");
    let result = launcher.check_game("1.20.1", "vanilla").await?;
    println!("{:#?}", result);
    Ok(())
}
```

Estos comandos están para devolver `true` o `false` dependiendo si una cosa está instalada o no.

No te olvides de haberlo importado ;)

```rust
use oxide_mc::OxideLauncher;
```

---
*Hecho con ❤️ por un desarrollador de 16 años.*