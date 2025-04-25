import React, { useState, useEffect } from 'react';
import Head from 'next/head';
import { Editor } from '@monaco-editor/react';
import { Bar } from 'react-chartjs-2';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  BarElement,
  Title,
  Tooltip,
  Legend,
} from 'chart.js';

// Register Chart.js components
ChartJS.register(
  CategoryScale,
  LinearScale,
  BarElement,
  Title,
  Tooltip,
  Legend
);

// Import interactive examples
import interactiveExamples from '../../../interactive_code_examples';

export default function TutorialPage() {
  const [activeExample, setActiveExample] = useState(0);
  const [anarchyCode, setAnarchyCode] = useState('');
  const [pythonCode, setPythonCode] = useState('');
  const [javascriptCode, setJavascriptCode] = useState('');
  const [activeTab, setActiveTab] = useState('anarchy');
  const [tokenData, setTokenData] = useState({
    labels: ['Anarchy Inference', 'Python', 'JavaScript'],
    datasets: [
      {
        label: 'Token Count',
        data: [0, 0, 0],
        backgroundColor: [
          'rgba(75, 192, 192, 0.6)',
          'rgba(54, 162, 235, 0.6)',
          'rgba(255, 206, 86, 0.6)',
        ],
        borderColor: [
          'rgba(75, 192, 192, 1)',
          'rgba(54, 162, 235, 1)',
          'rgba(255, 206, 86, 1)',
        ],
        borderWidth: 1,
      },
    ],
  });

  // List of examples
  const examples = [
    interactiveExamples.helloWorldExample,
    interactiveExamples.variableAssignmentExample,
    interactiveExamples.conditionalExample,
    interactiveExamples.functionExample,
    interactiveExamples.loopExample,
    interactiveExamples.arrayExample,
    interactiveExamples.objectExample,
    interactiveExamples.errorHandlingExample,
    interactiveExamples.stringExample,
    interactiveExamples.apiExample,
  ];

  // Update code and token data when active example changes
  useEffect(() => {
    const example = examples[activeExample];
    setAnarchyCode(example.anarchyCode.trim());
    setPythonCode(example.pythonCode.trim());
    setJavascriptCode(example.javascriptCode.trim());
    
    setTokenData({
      labels: ['Anarchy Inference', 'Python', 'JavaScript'],
      datasets: [
        {
          label: 'Token Count',
          data: [example.anarchyTokens, example.pythonTokens, example.javascriptTokens],
          backgroundColor: [
            'rgba(75, 192, 192, 0.6)',
            'rgba(54, 162, 235, 0.6)',
            'rgba(255, 206, 86, 0.6)',
          ],
          borderColor: [
            'rgba(75, 192, 192, 1)',
            'rgba(54, 162, 235, 1)',
            'rgba(255, 206, 86, 1)',
          ],
          borderWidth: 1,
        },
      ],
    });
  }, [activeExample]);

  // Calculate token savings
  const calculateSavings = () => {
    const example = examples[activeExample];
    const pythonSavings = Math.round((1 - example.anarchyTokens / example.pythonTokens) * 100);
    const jsSavings = Math.round((1 - example.anarchyTokens / example.javascriptTokens) * 100);
    return { pythonSavings, jsSavings };
  };

  const { pythonSavings, jsSavings } = calculateSavings();

  // Chart options
  const chartOptions = {
    responsive: true,
    plugins: {
      legend: {
        position: 'top',
      },
      title: {
        display: true,
        text: 'Token Count Comparison',
      },
      tooltip: {
        callbacks: {
          label: function(context) {
            return `${context.dataset.label}: ${context.raw} tokens`;
          }
        }
      }
    },
    scales: {
      y: {
        beginAtZero: true,
        title: {
          display: true,
          text: 'Token Count'
        }
      }
    }
  };

  return (
    <div className="min-h-screen bg-gray-50">
      <Head>
        <title>Anarchy Inference Interactive Tutorials</title>
        <meta name="description" content="Learn Anarchy Inference with interactive examples" />
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <header className="bg-gradient-to-r from-blue-600 to-indigo-700 text-white shadow-md">
        <div className="container mx-auto px-4 py-6">
          <h1 className="text-3xl font-bold">Anarchy Inference Interactive Tutorials</h1>
          <p className="mt-2">Learn the token-efficient programming language for LLMs</p>
        </div>
      </header>

      <main className="container mx-auto px-4 py-8">
        <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
          {/* Sidebar with example list */}
          <div className="lg:col-span-1 bg-white rounded-lg shadow-md p-4">
            <h2 className="text-xl font-semibold mb-4">Examples</h2>
            <ul className="space-y-2">
              {examples.map((example, index) => (
                <li key={index}>
                  <button
                    className={`w-full text-left px-3 py-2 rounded-md ${
                      activeExample === index
                        ? 'bg-blue-100 text-blue-700 font-medium'
                        : 'hover:bg-gray-100'
                    }`}
                    onClick={() => setActiveExample(index)}
                  >
                    {example.title}
                  </button>
                </li>
              ))}
            </ul>
          </div>

          {/* Main content area */}
          <div className="lg:col-span-3 space-y-6">
            {/* Example description */}
            <div className="bg-white rounded-lg shadow-md p-6">
              <h2 className="text-2xl font-bold mb-2">{examples[activeExample].title}</h2>
              <p className="text-gray-700 mb-4">{examples[activeExample].description}</p>
              
              <div className="bg-blue-50 border-l-4 border-blue-500 p-4 rounded">
                <h3 className="font-semibold text-blue-700 mb-2">Token Efficiency</h3>
                <p>
                  Anarchy Inference uses <span className="font-bold text-green-600">{pythonSavings}%</span> fewer tokens than Python 
                  and <span className="font-bold text-green-600">{jsSavings}%</span> fewer tokens than JavaScript 
                  for this example.
                </p>
              </div>
            </div>

            {/* Code editor tabs */}
            <div className="bg-white rounded-lg shadow-md overflow-hidden">
              <div className="flex border-b">
                <button
                  className={`px-4 py-2 ${
                    activeTab === 'anarchy'
                      ? 'bg-white border-b-2 border-blue-500 font-medium'
                      : 'bg-gray-100 hover:bg-gray-200'
                  }`}
                  onClick={() => setActiveTab('anarchy')}
                >
                  Anarchy Inference
                </button>
                <button
                  className={`px-4 py-2 ${
                    activeTab === 'python'
                      ? 'bg-white border-b-2 border-blue-500 font-medium'
                      : 'bg-gray-100 hover:bg-gray-200'
                  }`}
                  onClick={() => setActiveTab('python')}
                >
                  Python
                </button>
                <button
                  className={`px-4 py-2 ${
                    activeTab === 'javascript'
                      ? 'bg-white border-b-2 border-blue-500 font-medium'
                      : 'bg-gray-100 hover:bg-gray-200'
                  }`}
                  onClick={() => setActiveTab('javascript')}
                >
                  JavaScript
                </button>
              </div>
              <div className="h-80">
                {activeTab === 'anarchy' && (
                  <Editor
                    height="100%"
                    language="javascript" // Using JavaScript as a close approximation
                    theme="vs-dark"
                    value={anarchyCode}
                    onChange={setAnarchyCode}
                    options={{
                      minimap: { enabled: false },
                      fontSize: 14,
                      scrollBeyondLastLine: false,
                    }}
                  />
                )}
                {activeTab === 'python' && (
                  <Editor
                    height="100%"
                    language="python"
                    theme="vs-dark"
                    value={pythonCode}
                    onChange={setPythonCode}
                    options={{
                      minimap: { enabled: false },
                      fontSize: 14,
                      scrollBeyondLastLine: false,
                      readOnly: true,
                    }}
                  />
                )}
                {activeTab === 'javascript' && (
                  <Editor
                    height="100%"
                    language="javascript"
                    theme="vs-dark"
                    value={javascriptCode}
                    onChange={setJavascriptCode}
                    options={{
                      minimap: { enabled: false },
                      fontSize: 14,
                      scrollBeyondLastLine: false,
                      readOnly: true,
                    }}
                  />
                )}
              </div>
            </div>

            {/* Token visualization */}
            <div className="bg-white rounded-lg shadow-md p-6">
              <h3 className="text-xl font-semibold mb-4">Token Comparison</h3>
              <div className="h-64">
                <Bar data={tokenData} options={chartOptions} />
              </div>
              <div className="mt-4 bg-gray-50 p-4 rounded-md">
                <h4 className="font-medium mb-2">Explanation</h4>
                <p className="text-gray-700">{examples[activeExample].explanation}</p>
              </div>
            </div>
          </div>
        </div>
      </main>

      <footer className="bg-gray-800 text-white mt-12">
        <div className="container mx-auto px-4 py-6">
          <p className="text-center">
            © 2025 Anarchy Inference | <a href="https://github.com/APiTJLillo/Anarchy-Inference" className="text-blue-300 hover:underline">GitHub</a>
          </p>
        </div>
      </footer>
    </div>
  );
}
