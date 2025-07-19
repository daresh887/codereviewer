"use client";

import { useState } from "react";

interface RepoData {
  name: string;
  description: string;
  stargazers_count: number;
  forks_count: number;
  open_issues_count: number;
  html_url: string;
}

interface ErrorData {
  error: string;
}

export default function HomePage() {
  const [owner, setOwner] = useState("daresh887");
  const [repo, setRepo] = useState("codereviewer");
  const [repoData, setRepoData] = useState<RepoData | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);


  const fetchRepoInfo = async () => {
    setIsLoading(true);
    setRepoData(null);
    setError(null);

    const backendUrl = `/api/repo/${owner}/${repo}`;

    try {
      const response = await fetch(backendUrl);
      if (!response.ok) {
        const errorData: ErrorData = await response.json();
        throw new Error(errorData.error || `Request failed with status ${response.status}`);
      }
      const data: RepoData = await response.json();
      setRepoData(data);
    } catch (err) {
      if (err instanceof Error) {
        setError(err.message);
      }
    } finally {
      setIsLoading(false);
    }
  };

  const handleSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setError(null);



    if (owner && repo) {
      fetchRepoInfo();
    }
  };

  return (
    <main className="flex min-h-screen flex-col items-center p-8 bg-gray-50">
      <div className="w-full max-w-2xl">
        <h1 className="text-4xl font-bold text-center text-gray-800">
          Loro
        </h1>

        <form onSubmit={handleSubmit} className="mt-8 flex flex-col md:flex-row gap-4">
          <input
            type="text"
            value={owner}
            onChange={(e) => setOwner(e.target.value)}
            className={`flex-1 px-4 py-2 border rounded-md focus:outline-none focus:ring-2 ${
              'border-gray-300 focus:ring-blue-500 text-black'
            }`}
            required
          />
          <input
            type="text"
            value={repo}
            onChange={(e) => setRepo(e.target.value)}
            className={`flex-1 px-4 py-2 border rounded-md focus:outline-none focus:ring-2 ${
              'border-gray-300 focus:ring-blue-500 text-black'
            }`}
            required
          />
          <button
            type="submit"
            disabled={isLoading}
            className="px-6 py-2 bg-blue-600 text-white font-semibold rounded-md hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
          >
            {isLoading ? "Fetching..." : "Fetch Info"}
          </button>
        </form>

        <div className="mt-8 p-6 bg-white border border-gray-200 rounded-lg shadow-sm min-h-[150px] flex items-center justify-center">
          {isLoading && <p className="text-gray-500">Loading data...</p>}
          {error && <p className="text-red-500 font-semibold text-center">{error}</p>}
          {repoData && !error && (
             <div className="w-full">
              <h2 className="text-2xl font-bold text-gray-900">
                <a href={repoData.html_url} target="_blank" rel="noopener noreferrer" className="hover:underline">
                  {repoData.name}
                </a>
              </h2>
              <p className="text-gray-600 mt-2">{repoData.description}</p>
              <div className="mt-4 flex flex-wrap gap-x-6 gap-y-2 text-gray-700">
                <span>⭐ <strong>{repoData.stargazers_count.toLocaleString()}</strong> Stars</span>
                <span>🍴 <strong>{repoData.forks_count.toLocaleString()}</strong> Forks</span>
                <span>🐞 <strong>{repoData.open_issues_count.toLocaleString()}</strong> Open Issues</span>
              </div>
            </div>
          )}
          {!isLoading && !error && !repoData && (
            <p className="text-gray-400">Enter a repository to get started.</p>
          )}
        </div>
      </div>
    </main>
  );
}
