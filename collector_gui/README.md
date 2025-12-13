# Collector GUI - Iced Edition

Application GUI de collection d'artefacts forensiques, utilisant Iced 0.13.1.

## Fonctionnalités

- Sélection de dossiers source et destination via dialogues natifs
- Option VSS (Windows) : Extraction Volume Shadow Copy
- Compression ZIP avec mot de passe optionnel
- Gestion des ressources : recherche, filtrage par catégorie, sélection multiple
- Filtre "Selected only" pour voir uniquement les ressources cochées
- Thème adaptatif : détection auto + toggle manuel (clair/sombre)
- Icônes vectorielles : Lucide Icons en SVG

## Note importante : DPI Scaling

Cette version utilise `scale_factor(|_| 1.0)` pour forcer le rendu à 100%.
Cela évite les problèmes d'alignement des icônes SVG sur les écrans avec 
scaling Windows (125%, 150%, etc.).

L'interface sera légèrement plus petite sur les écrans haute résolution,
mais toutes les icônes s'afficheront correctement.

## Dépendances principales

```toml
[dependencies]
iced = { version = "0.13.1", features = ["tokio", "advanced", "svg", "multi-window"] }
tokio = { version = "1.48", features = ["full"] }
rfd = "0.16"
dark-light = "2.0"
serde = { version = "1.0", features = ["derive"] }
toml = "0.9.8"
```

## Compilation

```bash
cargo build --release
```

## Structure du projet

```
src/
├── main.rs              # Application principale (avec scale_factor)
├── com/                 # Logique métier
│   ├── mod.rs
│   ├── collection.rs    # Exécution de la collection
│   ├── config.rs        # Configuration TOML
│   └── resources.rs     # Gestion des ressources
├── gui/                 # Interface Iced
│   ├── mod.rs
│   ├── app.rs           # État et logique de l'app
│   └── message.rs       # Messages/événements
├── style/               # Styles et thèmes
│   ├── mod.rs
│   ├── icons.rs         # Icônes SVG Lucide
│   └── theme.rs         # Couleurs et styles
├── utils/               # Utilitaires
│   ├── mod.rs
│   ├── values_linux.rs
│   └── values_windows.rs
└── views/               # Vues de l'interface
    ├── mod.rs
    ├── footer.rs
    ├── input_section.rs
    ├── modal.rs
    ├── output_section.rs
    ├── resources_section.rs
    └── resources_table.rs
```

## Icônes Lucide

Les icônes sont incluses en SVG depuis `assets/icons/`.
Pour ajouter une icône :

1. Copier le SVG depuis [lucide.dev](https://lucide.dev/)
2. Créer `assets/icons/nom-icone.svg`
3. Ajouter dans `src/style/icons.rs` :
```rust
pub const NOM_ICONE: &[u8] = include_icon!("nom-icone");
```

## Licence

MIT
