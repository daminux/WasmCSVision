# CSV Analyzer

Un analyseur de fichiers CSV en temps réel utilisant WebAssembly (Rust) et JavaScript. Cette application web permet d'analyser rapidement et efficacement de grands fichiers CSV en détectant automatiquement les types de données, les statistiques de colonnes et d'autres métriques utiles.

## Fonctionnalités

- 🚀 Analyse rapide grâce à WebAssembly (Rust)
- 📊 Détection automatique des types de données
- 🔍 Analyse statistique des colonnes
- ⚡ Support de l'échantillonnage pour les grands fichiers
- 🎯 Détection automatique du délimiteur
- 📈 Affichage des métriques de performance

### Types de données supportés

- Entiers
- Décimaux
- Booléens
- Dates
- Date et heure
- Heure
- Email
- URL
- Adresses IP
- Texte

## Prérequis

- [Node.js](https://nodejs.org/) (v14 ou supérieur)
- [Rust](https://www.rust-lang.org/) et [cargo](https://doc.rust-lang.org/cargo/)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

## Installation et démarrage rapide

1. Clonez le dépôt :
```bash
git clone [URL_DU_REPO]
cd csv-analyzer
```

2. Installez les dépendances :
```bash
npm install
```

3. Lancez le build complet (WASM + web) :
```bash
npm run build
```

4. Pour développer, utilisez le serveur de développement :
```bash
npm run dev
```

L'application sera disponible à l'adresse `http://localhost:3000`.

## Structure du projet

```
.
├── wasm/              # Code Rust/WebAssembly
│   ├── src/
│   │   └── lib.rs     # Code Rust pour l'analyse
│   ├── pkg/           # Build WebAssembly
│   └── Cargo.toml     # Configuration Rust
├── src/               # Code source web
│   ├── index.js       # Point d'entrée JavaScript
│   └── index.html     # Template HTML
├── dist/              # Dossier de build production
├── package.json       # Configuration npm
└── webpack.config.js  # Configuration Webpack
```

## Utilisation

1. Ouvrez l'application dans votre navigateur
2. Sélectionnez un fichier CSV à analyser
3. (Optionnel) Configurez l'échantillonnage si nécessaire
4. Cliquez sur "Analyser" pour lancer l'analyse
5. Consultez les résultats détaillés dans le tableau

## Fonctionnalités détaillées

### Analyse des colonnes
Pour chaque colonne, l'analyseur fournit :
- Type de données détecté
- Niveau de confiance dans la détection
- Nombre de valeurs analysées
- Sous-types détectés
- Exemples de valeurs
- Statistiques (min/max, longueurs, valeurs nulles)

### Configuration de l'échantillonnage
- Possibilité d'activer/désactiver l'échantillonnage
- Configuration du nombre maximum de valeurs à analyser par colonne
- Indication du mode d'analyse dans les résultats

## Performances

L'utilisation de WebAssembly (Rust) permet :
- Une analyse rapide même sur de grands fichiers
- Une consommation mémoire optimisée
- Des performances proches du natif
- Un affichage en temps réel des métriques de performance

## Développement

### Scripts disponibles

- `npm run dev` : Démarre le serveur de développement avec hot-reload
- `npm run build` : Compile l'application complète (WASM + web) pour la production
- `npm run build:wasm` : Compile uniquement le code Rust en WebAssembly
- `npm run clean` : Nettoie les fichiers générés

### Développement Rust/WASM

1. Modifiez le code dans `wasm/src/lib.rs`
2. Le serveur de développement recompilera automatiquement les changements
3. Pour un build production, utilisez `npm run build`

## License

[MIT License](LICENSE)