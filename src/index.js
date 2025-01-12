import init, { AnalyzerConfig, CSVAnalyzer } from '../wasm/pkg/csv_analyzer_wasm.js';

let analyzer = null;

async function initializeWasm() {
    try {
        await init();
        const config = new AnalyzerConfig();
        // Par défaut, pas d'échantillonnage (analyse complète)
        config.sample_size = null;
        analyzer = new CSVAnalyzer(config);
        console.log('WASM initialized');
    } catch (error) {
        console.error('WASM initialization error:', error);
    }
}

function getTypeLabel(type) {
    const labels = {
        'integer': 'Entier',
        'float': 'Décimal',
        'boolean': 'Booléen',
        'date': 'Date',
        'datetime': 'Date et heure',
        'time': 'Heure',
        'email': 'Email',
        'url': 'URL',
        'ip': 'Adresse IP',
        'string': 'Texte',
        'null': 'Nul'
    };
    return labels[type] || type;
}

function formatConfidence(confidence) {
    return (confidence * 100).toFixed(1) + '%';
}

function formatExecutionTime(startTime) {
    const endTime = performance.now();
    const executionTime = endTime - startTime;
    return executionTime < 1000 
        ? `${Math.round(executionTime)}ms` 
        : `${(executionTime / 1000).toFixed(2)}s`;
}

function formatFileSize(bytes) {
    if (bytes === 0) return '0 Bytes';
    
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function formatAnalyzedValues(analyzed, total) {
    if (analyzed === total) {
        return `${analyzed} (100%)`;
    }
    const percentage = ((analyzed / total) * 100).toFixed(1);
    return `${analyzed} (${percentage}%)`;
}

// Rendre les fonctions accessibles globalement
window.exportToCSV = function(analysis) {
    // En-têtes du CSV (dans le même ordre que le tableau)
    const headers = [
        'Colonne',
        'Type',
        'Confiance',
        'Valeurs analysées',
        'Sous-types',
        'Exemples',
        'Valeurs totales',
        'Valeurs uniques',
        'Valeurs nulles',
        'Min',
        'Max',
        'Longueur Min',
        'Longueur Max'
    ].join(';');

    // Données des colonnes (dans le même ordre que le tableau)
    const rows = analysis.columns.map(stats => [
        stats.name,
        getTypeLabel(stats.type_name),
        formatConfidence(stats.type_details.confidence),
        formatAnalyzedValues(stats.analyzed_count, stats.total_count),
        stats.type_details.subtypes.map(subtype => getTypeLabel(subtype)).join(', '),
        stats.type_details.format_examples.join(', '),
        stats.total_count,
        stats.unique_values,
        stats.null_count,
        stats.min_value || '',
        stats.max_value || '',
        stats.min_length,
        stats.max_length
    ].join(';'));

    // Assemblage du CSV complet avec BOM pour Excel
    return '\ufeff' + [headers, ...rows].join('\n');
}

window.downloadCSV = function(csvContent, fileName) {
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const link = document.createElement('a');
    const url = URL.createObjectURL(blob);
    link.setAttribute('href', url);
    link.setAttribute('download', fileName);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
}

function displayResults(analysis, startTime, fileSize) {
    const results = document.getElementById('results');
    console.log('Starting to display results:', analysis);
    
    const executionTime = formatExecutionTime(startTime);
    
    let html = `
        <div class="results">
            <div class="actions">
                <button id="downloadButton" class="download-btn">
                    Télécharger l'analyse en CSV
                </button>
            </div>
            <h2>Résumé</h2>
            <p>Nombre de lignes: ${analysis.row_count}</p>
            <p>Nombre de colonnes: ${analysis.column_count}</p>
            <p>Taille du fichier: ${formatFileSize(fileSize)}</p>
            <p class="execution-time">Temps d'analyse: ${executionTime}</p>
            <p>Vitesse de traitement: ${formatFileSize(fileSize/(parseFloat(executionTime)/1000))}/s</p>
            <p>Mode d'analyse: ${analysis.sample_size ? `Échantillonnage (${analysis.sample_size} valeurs max)` : 'Analyse complète'}</p>
            
            <h2>Analyse des colonnes</h2>
            <table>
                <thead>
                    <tr>
                        <th>Colonne</th>
                        <th>Type</th>
                        <th>Confiance</th>
                        <th>Valeurs analysées</th>
                        <th>Sous-types</th>
                        <th>Exemples</th>
                        <th>Valeurs totales</th>
                        <th>Valeurs uniques</th>
                        <th>Valeurs nulles</th>
                        <th>Min</th>
                        <th>Max</th>
                        <th>Longueur Min</th>
                        <th>Longueur Max</th>
                    </tr>
                </thead>
                <tbody>
    `;

    analysis.columns.forEach(stats => {
        const subtypesHtml = stats.type_details.subtypes
            .map(subtype => getTypeLabel(subtype))
            .join(', ');
        
        const examplesHtml = stats.type_details.format_examples
            .slice(0, 3)
            .join(', ');

        html += `
            <tr>
                <td>${stats.name}</td>
                <td>${getTypeLabel(stats.type_name)}</td>
                <td>${formatConfidence(stats.type_details.confidence)}</td>
                <td>${formatAnalyzedValues(stats.analyzed_count, stats.total_count)}</td>
                <td>${subtypesHtml}</td>
                <td>${examplesHtml}</td>
                <td>${stats.total_count}</td>
                <td>${stats.unique_values}</td>
                <td>${stats.null_count}</td>
                <td>${stats.min_value || '-'}</td>
                <td>${stats.max_value || '-'}</td>
                <td>${stats.min_length}</td>
                <td>${stats.max_length}</td>
            </tr>
        `;
    });

    html += `
                </tbody>
            </table>
        </div>
    `;

    console.log('Generated HTML:', html);
    results.innerHTML = html;
    
    // Ajouter l'écouteur d'événement pour le bouton de téléchargement
    const downloadButton = document.getElementById('downloadButton');
    if (downloadButton) {
        downloadButton.addEventListener('click', () => {
            downloadCSV(exportToCSV(analysis), "analyse_colonnes.csv");
        });
    }
}

function setupUI() {
    const fileInput = document.getElementById('fileInput');
    const fileInfo = document.getElementById('fileInfo');
    const analyzeBtn = document.getElementById('analyzeBtn');
    const results = document.getElementById('results');
    const sampleSizeInput = document.getElementById('sampleSize');
    const sampleSizeToggle = document.getElementById('sampleSizeToggle');

    // Ajout des éléments de configuration d'échantillonnage
    const configDiv = document.createElement('div');
    configDiv.className = 'config-section';
    configDiv.innerHTML = `
        <label>
            <input type="checkbox" id="sampleSizeToggle">
            Utiliser l'échantillonnage
        </label>
        <input type="number" id="sampleSize" value="1000" min="100" disabled>
        <span>valeurs par colonne</span>
    `;
    fileInput.parentNode.insertBefore(configDiv, fileInput.nextSibling);

    // Gestion de l'échantillonnage
    document.getElementById('sampleSizeToggle').addEventListener('change', function(e) {
        const sampleSizeInput = document.getElementById('sampleSize');
        sampleSizeInput.disabled = !e.target.checked;
        
        const config = new AnalyzerConfig();
        if (e.target.checked) {
            config.sample_size = parseInt(sampleSizeInput.value);
        } else {
            config.sample_size = null;
        }
        analyzer = new CSVAnalyzer(config);
    });

    document.getElementById('sampleSize').addEventListener('change', function(e) {
        if (document.getElementById('sampleSizeToggle').checked) {
            const config = new AnalyzerConfig();
            config.sample_size = parseInt(e.target.value);
            analyzer = new CSVAnalyzer(config);
        }
    });

    fileInput.addEventListener('change', (e) => {
        const file = e.target.files[0];
        if (!file) return;

        fileInfo.textContent = `Fichier sélectionné : ${file.name}`;
        fileInfo.style.display = 'block';
        analyzeBtn.disabled = false;
    });

    analyzeBtn.addEventListener('click', async () => {
        const fileInput = document.getElementById('fileInput');
        const file = fileInput.files[0];
        if (!file || !analyzer) return;

        try {
            const startTime = performance.now();
            console.log('Starting analysis of file:', file.name);
            analyzeBtn.disabled = true;
            results.innerHTML = '<div class="loading">Analyse en cours...</div>';

            const content = await file.text();
            console.log('File content loaded, length:', content.length);
            
            const analysis = analyzer.analyze(content);
            console.log('Analysis result:', analysis);
            
            displayResults(analysis, startTime, file.size);
        } catch (error) {
            console.error('Analysis error:', error);
            results.innerHTML = `<div class="error">Erreur lors de l'analyse: ${error.message}</div>`;
        } finally {
            analyzeBtn.disabled = false;
        }
    });
}

const styles = document.createElement('style');
styles.textContent = `
    .container {
        max-width: 1200px;
        margin: 0 auto;
        padding: 20px;
    }

    .config-section {
        margin: 15px 0;
        padding: 10px;
        border: 1px solid #ddd;
        border-radius: 4px;
        background-color: #f8f8f8;
    }

    .config-section input[type="number"] {
        width: 100px;
        margin: 0 10px;
        padding: 5px;
    }

    .results table {
        width: 100%;
        border-collapse: collapse;
        margin-top: 20px;
        font-size: 14px;
    }

    .results th, .results td {
        border: 1px solid #ddd;
        padding: 8px;
        text-align: left;
    }

    .results th {
        background-color: #f5f5f5;
        white-space: nowrap;
    }

    .results td {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 200px;
    }

    .loading {
        padding: 20px;
        text-align: center;
        color: #666;
    }

    .error {
        color: red;
        padding: 10px;
        border: 1px solid red;
        border-radius: 4px;
        margin-top: 10px;
    }

    .execution-time {
        font-weight: bold;
        color: #2c5282;
        margin: 10px 0;
    }

    .actions {
        margin: 20px 0;
    }

    .download-btn {
        background-color: #4CAF50;
        border: none;
        color: white;
        padding: 10px 20px;
        text-align: center;
        text-decoration: none;
        display: inline-block;
        font-size: 16px;
        margin: 4px 2px;
        cursor: pointer;
        border-radius: 4px;
        transition: background-color 0.3s;
    }

    .download-btn:hover {
        background-color: #45a049;
    }
`;

document.head.appendChild(styles);

document.addEventListener('DOMContentLoaded', async () => {
    console.log('Page loaded, initializing...');
    await initializeWasm();
    setupUI();
});