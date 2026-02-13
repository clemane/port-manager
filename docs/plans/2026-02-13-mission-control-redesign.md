# Mission Control — Port Manager UI/UX Redesign

## Direction

Dashboard cockpit de monitoring DevOps. Dense en information, indicateurs temps reel, micro-animations de statut. Ambiance Grafana/Datadog/Lens.

## Typographie

- **Display/Titres** : Sora (Google Fonts) — geometrique, moderne
- **Body/UI** : DM Sans (Google Fonts) — clean, lisible
- **Data/Mono** : JetBrains Mono — ports, IPs, URLs, process names

## Palette Dark (theme principal)

| Token | Valeur | Usage |
|-------|--------|-------|
| bg | `#06080f` | Background principal (bleute profond) |
| surface | `#0d1117` | Cards, panels |
| surface-elevated | `#161b22` | Modals, dropdowns, inputs |
| border | `#21262d` | Bordures subtiles |
| accent | `#58a6ff` | Actions, liens, indicateur actif |
| success | `#3fb950` | Running, OK |
| danger | `#f85149` | Erreurs |
| warning | `#d29922` | Alertes |
| text-primary | `#e6edf3` | Texte principal |
| text-secondary | `#8b949e` | Texte secondaire |
| text-muted | `#484f58` | Labels discrets |

Gradient ambiance : `radial-gradient(ellipse at 50% 0%, rgba(88,166,255,0.03) 0%, transparent 70%)`

## Layout

### Sidebar collapsible

- **Collapsed** (defaut) : 56px, icones SVG seulement
- **Expanded** : 200px, icones + labels
- **Trigger** : Bouton hamburger ou hover 300ms
- **Indicateur actif** : Barre verticale 3px accent a gauche
- **Footer** : Theme switcher (icone cycle en collapsed)
- Transition : `width 200ms ease-out`, labels fade 100ms

### Header contextuel

- Hauteur : 56px, fixe en haut du contenu
- Gauche : Titre page (Sora 18px semibold)
- Centre : Breadcrumb contextuel
- Droite : Actions rapides (refresh, filtres, compteur)

### Status bar enrichie

- Hauteur : 32px
- Indicateurs live avec dots animes (pulse vert, orange warning)
- Metriques cliquables : `3 forwards` / `142 ports` / `1 tunnel`
- Droite : Version + dot connexion backend

### Transitions de page

- Sortie : `opacity 1->0` + `translateY(0->-4px)` en 100ms
- Entree : `opacity 0->1` + `translateY(4px->0)` en 150ms ease-out
- Via `<Transition>` Vue Router

## Nouveaux composants

### PmMetricCard

- Grille 3-4 cards en ligne
- Icone SVG + valeur grande (Sora 28px bold) + label (DM Sans 12px muted)
- Bordure gauche 3px coloree selon type
- Count-up anime au chargement (400ms)

### PmSkeletonLoader

- Barres 12px, coins arrondis
- Animation shimmer (gradient translate gauche->droite, 1.5s infinite)
- Variantes : table (5 lignes, largeurs variees), card, input

### PmStatusDot

- Running : pulse (scale 1->1.4->1, cycle 2s)
- Error : blink (opacity 1->0.3->1, cycle 1s)
- Stopped : statique, opacite 50%
- Taille : 8px, border-radius 50%

## Composants redesignes

### PmTable v2

- Header : fond surface-elevated, uppercase 11px, letter-spacing 0.05em, sticky
- Lignes : hauteur 44px, hover slide-effect (translateX 2px)
- Status LED : dot 8px anime + texte
- Staggered reveal : 20ms delay entre lignes, max 15
- Skeleton loading au lieu de "Loading..."
- Pagination compacte : "1-50 of 142" + chevrons

### PmButton v2

- Primary : gradient subtil, shadow accent 15%
- Ghost : hover pop effect (scale 0.95->1)
- Danger : pulse confirmation au hover
- Tous : radius 6px, transitions 150ms, focus ring 2px

### PmInput v2

- Hauteur 36px, fond surface-elevated
- Focus : border accent + glow (0 0 0 3px accent/15%)
- Search : icone loupe SVG integree a gauche

### PmModal v2

- Overlay : backdrop-filter blur(4px), noir 60%
- Entree : scale(0.95)->scale(1) + opacity, 200ms
- Spacing augmente, border-bottom header supprime

### PmBadge v2

- Compact : padding 2px 8px, font 11px
- Dot colore 6px avant le texte
- Border subtile 1px couleur badge 20%

### PmToast v2

- Position : bas-droite
- Barre progress countdown en bas
- Icone type a gauche

## Vues redesignees

### Dashboard

- Metric cards : Listening (vert), Established (bleu), Conflicts (rouge), Total (neutre)
- Filtres en ligne : search + state select + protocol select sur fond surface
- Table avec status LEDs, lignes conflit surlignees danger 5%
- Empty state SVG

### K8s Browser (refonte 3 colonnes)

- **Col 1** (~200px) : Clusters & Namespaces, expandables
- **Col 2** (~250px) : Resources TreeView, icones Services/Pods
- **Col 3** (reste) : Details + ports + bouton Forward inline
- Bordures verticales, fade transitions entre contenus
- Plus de dropdowns cascades

### Forwards

- Metric cards : Active (vert), Errored (rouge), Favorites (bleu)
- Favorites : grille 2-3 colonnes, cards compactes, groupes par separateurs
- Active : table avec icone etoile, URL mono cliquable + copy

### Ngrok

- Launch tunnel : card proeminente surface-elevated, border accent, layout horizontal
- Active tunnels : table avec URL mono, status LED
- Reserved domains : chips/tags avec X delete, input ajout inline

### Settings

- Sections en cards distinctes avec titre + description
- Theme : miniatures couleur a cote des boutons
- Credentials : toggle visibility (oeil)
- Kubeconfigs : cards au lieu de lignes

## Adaptation themes

### Light

- bg `#f6f8fa`, surface `#ffffff`, surface-elevated `#f0f3f6`
- border `#d0d7de`, accent `#0969da`
- text-primary `#1f2328`, secondary `#656d76`
- Shadow augmente, sidebar fond blanc + bordure droite
- Gradient ambiance bleu `rgba(9,105,218,0.04)`

### Cyberpunk

- Structure Mission Control + neon glows
- Accent bars : box-shadow glow cyan/magenta
- Status dots : glow neon (0 0 6px)
- Double gradient cyan+magenta en coins opposes
- Background noir `#000000`

### Matrix

- Structure Mission Control + canvas rain
- Accent `#00ff41`, tout monochrome vert
- Metric cards tout en JetBrains Mono
- Bordures vert fonce 10%, radius 2px
- Sidebar barre verte + glow `0 0 8px rgba(0,255,65,0.3)`
- Gradient ambiance vert `rgba(0,255,65,0.02)`

## Nouvelles variables CSS

```
--pm-surface-elevated
--pm-accent-glow
--pm-gradient-ambiance
--pm-status-pulse-color
```

## Animations

- Toutes GPU-accelerated (transform + opacity uniquement)
- `will-change` sur elements animes frequemment
- `prefers-reduced-motion` : toutes animations desactivees
- CSS pur + transitions Vue, zero librairie

## Fichiers impactes

### Nouveaux

- `PmMetricCard.vue`
- `PmSkeletonLoader.vue`
- `PmStatusDot.vue`

### Modifies

- Layout : `AppLayout.vue`, `PmSidebar.vue`, `PmStatusBar.vue`
- UI : `PmTable.vue`, `PmButton.vue`, `PmInput.vue`, `PmModal.vue`, `PmBadge.vue`, `PmToast.vue`, `PmThemeSwitcher.vue`
- Views : `DashboardView.vue`, `K8sBrowserView.vue`, `ForwardsView.vue`, `NgrokView.vue`, `SettingsView.vue`
- Tokens : `colors.css`, `semantic.css`, `dark.css`, `light.css`, `cyberpunk.css`, `matrix.css`
- Global : `index.html` (fonts), `style.css` (keyframes, reset)

### Non touches

- Composables, types, router, backend Rust
