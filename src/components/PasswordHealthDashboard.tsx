import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { 
  AlertCircle, 
  CheckCircle, 
  RefreshCw,
  Download,
  BarChart3,
  Users,
  Calendar,
  Info,
  TrendingDown,
  Eye,
  EyeOff
} from "lucide-react";

interface DashboardData {
  security_score: {
    total_score: number;
    strength_score: number;
    uniqueness_score: number;
    age_score: number;
    breach_score: number;
    recommendations: string[];
    weak_passwords: number;
    reused_passwords: number;
    breached_passwords: number;
    old_passwords: number;
  };
  password_analyses: Record<string, any>;
  reused_groups: Array<{
    password_hash: string;
    entries: string[];
    risk_level: 'Low' | 'Medium' | 'High' | 'Critical';
  }>;
  breach_results: Record<string, any>;
  age_distribution: Record<string, number>;
  generated_at: string;
}

interface PasswordHealthDashboardProps {
  onBack: () => void;
}

export default function PasswordHealthDashboard({ onBack }: PasswordHealthDashboardProps) {
  const [dashboardData, setDashboardData] = useState<DashboardData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'overview' | 'details' | 'reused' | 'breaches' | 'age'>('overview');
  const [showReport, setShowReport] = useState(false);
  const [reportData, setReportData] = useState<string>('');

  useEffect(() => {
    loadDashboardData();
  }, []);

  const loadDashboardData = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await invoke<DashboardData>("generate_password_health_dashboard");
      setDashboardData(data);
    } catch (err) {
      setError(err as string);
    } finally {
      setLoading(false);
    }
  };

  const generateReport = async () => {
    try {
      const report = await invoke<string>("generate_dashboard_report");
      setReportData(report);
      setShowReport(true);
    } catch (err) {
      setError(err as string);
    }
  };

  const exportJSON = async () => {
    try {
      const jsonData = await invoke<string>("export_dashboard_json");
      const blob = new Blob([jsonData], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `password-health-${new Date().toISOString().split('T')[0]}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      setError(err as string);
    }
  };

  const exportCSV = async () => {
    try {
      const csvData = await invoke<string>("export_metrics_csv");
      const blob = new Blob([csvData], { type: 'text/csv' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `password-metrics-${new Date().toISOString().split('T')[0]}.csv`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      setError(err as string);
    }
  };

  const getScoreColor = (score: number) => {
    if (score >= 90) return "text-green-600 bg-green-50 border-green-200";
    if (score >= 70) return "text-yellow-600 bg-yellow-50 border-yellow-200";
    if (score >= 50) return "text-orange-600 bg-orange-50 border-orange-200";
    return "text-red-600 bg-red-50 border-red-200";
  };

  const getScoreIcon = (score: number) => {
    if (score >= 90) return <CheckCircle className="h-5 w-5 text-green-500" />;
    if (score >= 70) return <Info className="h-5 w-5 text-yellow-500" />;
    return <AlertCircle className="h-5 w-5 text-red-500" />;
  };

  const getRiskIcon = (level: string) => {
    switch (level) {
      case 'Critical': return <AlertCircle className="h-4 w-4 text-red-500" />;
      case 'High': return <AlertCircle className="h-4 w-4 text-orange-500" />;
      case 'Medium': return <Info className="h-4 w-4 text-yellow-500" />;
      default: return <CheckCircle className="h-4 w-4 text-green-500" />;
    }
  };

  const renderProgressBar = (score: number) => {
    const percentage = Math.max(0, Math.min(100, score));
    const barColor = score >= 90 ? 'bg-green-500' : score >= 70 ? 'bg-yellow-500' : score >= 50 ? 'bg-orange-500' : 'bg-red-500';
    
    return (
      <div className="w-full bg-gray-200 rounded-full h-2">
        <div 
          className={`h-2 rounded-full transition-all duration-300 ${barColor}`}
          style={{ width: `${percentage}%` }}
        ></div>
      </div>
    );
  };

  const renderAgeChart = (distribution: Record<string, number>) => {
    const total = Object.values(distribution).reduce((sum, count) => sum + count, 0);
    if (total === 0) return <div className="text-gray-500">No data available</div>;

    return (
      <div className="space-y-3">
        {Object.entries(distribution).map(([range, count]) => {
          const percentage = (count / total) * 100;
          return (
            <div key={range} className="flex items-center">
              <div className="w-24 text-sm text-gray-600">{range}</div>
              <div className="flex-1 mx-3">
                <div className="w-full bg-gray-200 rounded-full h-4">
                  <div 
                    className="bg-blue-500 h-4 rounded-full transition-all duration-300"
                    style={{ width: `${percentage}%` }}
                  ></div>
                </div>
              </div>
              <div className="w-16 text-sm font-medium text-gray-900">{count}</div>
            </div>
          );
        })}
      </div>
    );
  };

  if (loading) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-center">
          <RefreshCw className="h-8 w-8 text-primary-600 animate-spin mx-auto mb-4" />
          <div className="text-lg font-medium text-gray-900">Analyzing Password Health...</div>
          <div className="text-sm text-gray-500">This may take a moment</div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-center max-w-md">
          <AlertCircle className="h-12 w-12 text-red-500 mx-auto mb-4" />
          <div className="text-lg font-medium text-gray-900 mb-2">Analysis Failed</div>
          <div className="text-sm text-gray-500 mb-4">{error}</div>
          <button
            onClick={loadDashboardData}
            className="btn btn-primary"
          >
            Try Again
          </button>
        </div>
      </div>
    );
  }

  if (!dashboardData) return null;

  const { security_score } = dashboardData;
  const totalPasswords = Object.keys(dashboardData.password_analyses).length;

  return (
    <div className="h-full flex flex-col bg-gray-50">
      {/* Header */}
      <header className="bg-white border-b border-gray-200 px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <button
              onClick={onBack}
              className="text-gray-500 hover:text-gray-700"
            >
              ←
            </button>
            <div>
              <h1 className="text-xl font-semibold text-gray-900">Password Health Dashboard</h1>
              <p className="text-sm text-gray-500">
                Analysis generated on {new Date(dashboardData.generated_at).toLocaleString()}
              </p>
            </div>
          </div>
          
          <div className="flex items-center space-x-3">
            <button
              onClick={generateReport}
              className="btn btn-secondary flex items-center space-x-2"
            >
              {showReport ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
              <span>{showReport ? 'Hide' : 'Show'} Report</span>
            </button>
            <button
              onClick={exportJSON}
              className="btn btn-secondary flex items-center space-x-2"
            >
              <Download className="h-4 w-4" />
              <span>Export JSON</span>
            </button>
            <button
              onClick={exportCSV}
              className="btn btn-secondary flex items-center space-x-2"
            >
              <Download className="h-4 w-4" />
              <span>Export CSV</span>
            </button>
            <button
              onClick={loadDashboardData}
              className="btn btn-primary flex items-center space-x-2"
            >
              <RefreshCw className="h-4 w-4" />
              <span>Refresh</span>
            </button>
          </div>
        </div>
      </header>

      {/* Show Report Modal */}
      {showReport && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg max-w-4xl max-h-[80vh] overflow-hidden">
            <div className="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
              <h2 className="text-lg font-semibold">Security Report</h2>
              <button
                onClick={() => setShowReport(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                ✕
              </button>
            </div>
            <div className="p-6 overflow-y-auto max-h-96">
              <pre className="text-xs whitespace-pre-wrap font-mono bg-gray-50 p-4 rounded">
                {reportData}
              </pre>
            </div>
          </div>
        </div>
      )}

      {/* Content */}
      <div className="flex-1 overflow-auto">
        {/* Overall Score Card */}
        <div className="p-6">
          <div className={`rounded-lg border-2 p-6 ${getScoreColor(security_score.total_score)}`}>
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center space-x-3">
                {getScoreIcon(security_score.total_score)}
                <div>
                  <h2 className="text-lg font-semibold">Overall Security Score</h2>
                  <p className="text-sm opacity-75">Based on {totalPasswords} passwords</p>
                </div>
              </div>
              <div className="text-right">
                <div className="text-3xl font-bold">{security_score.total_score}/100</div>
                <div className="text-sm opacity-75">
                  {security_score.total_score >= 90 ? 'Excellent' : 
                   security_score.total_score >= 70 ? 'Good' : 
                   security_score.total_score >= 50 ? 'Fair' : 'Poor'}
                </div>
              </div>
            </div>
            
            {/* Component Scores */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div className="text-center">
                <div className="text-lg font-semibold">{security_score.strength_score}%</div>
                <div className="text-xs opacity-75">Strength</div>
                {renderProgressBar(security_score.strength_score)}
              </div>
              <div className="text-center">
                <div className="text-lg font-semibold">{security_score.uniqueness_score}%</div>
                <div className="text-xs opacity-75">Uniqueness</div>
                {renderProgressBar(security_score.uniqueness_score)}
              </div>
              <div className="text-center">
                <div className="text-lg font-semibold">{security_score.age_score}%</div>
                <div className="text-xs opacity-75">Age</div>
                {renderProgressBar(security_score.age_score)}
              </div>
              <div className="text-center">
                <div className="text-lg font-semibold">{security_score.breach_score}%</div>
                <div className="text-xs opacity-75">Breach Status</div>
                {renderProgressBar(security_score.breach_score)}
              </div>
            </div>
          </div>
        </div>

        {/* Quick Stats */}
        <div className="px-6 pb-6">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="bg-white rounded-lg border border-gray-200 p-4">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-2xl font-bold text-red-600">{security_score.weak_passwords}</div>
                  <div className="text-sm text-gray-600">Weak Passwords</div>
                </div>
                <TrendingDown className="h-8 w-8 text-red-500" />
              </div>
            </div>
            
            <div className="bg-white rounded-lg border border-gray-200 p-4">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-2xl font-bold text-orange-600">{security_score.reused_passwords}</div>
                  <div className="text-sm text-gray-600">Reused Passwords</div>
                </div>
                <Users className="h-8 w-8 text-orange-500" />
              </div>
            </div>
            
            <div className="bg-white rounded-lg border border-gray-200 p-4">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-2xl font-bold text-red-600">{security_score.breached_passwords}</div>
                  <div className="text-sm text-gray-600">Breached Passwords</div>
                </div>
                <AlertCircle className="h-8 w-8 text-red-500" />
              </div>
            </div>
            
            <div className="bg-white rounded-lg border border-gray-200 p-4">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-2xl font-bold text-yellow-600">{security_score.old_passwords}</div>
                  <div className="text-sm text-gray-600">Old Passwords</div>
                </div>
                <Calendar className="h-8 w-8 text-yellow-500" />
              </div>
            </div>
          </div>
        </div>

        {/* Tabs */}
        <div className="px-6 pb-6">
          <div className="bg-white rounded-lg border border-gray-200">
            {/* Tab Navigation */}
            <div className="border-b border-gray-200">
              <nav className="flex space-x-8 px-6 py-3">
                {[
                  { key: 'overview', label: 'Overview', icon: BarChart3 },
                  { key: 'reused', label: 'Reused Passwords', icon: Users },
                  { key: 'age', label: 'Password Age', icon: Calendar }
                ].map(({ key, label, icon: Icon }) => (
                  <button
                    key={key}
                    onClick={() => setActiveTab(key as any)}
                    className={`flex items-center space-x-2 py-2 px-1 border-b-2 font-medium text-sm ${
                      activeTab === key
                        ? 'border-primary-500 text-primary-600'
                        : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                    }`}
                  >
                    <Icon className="h-4 w-4" />
                    <span>{label}</span>
                  </button>
                ))}
              </nav>
            </div>

            {/* Tab Content */}
            <div className="p-6">
              {activeTab === 'overview' && (
                <div className="space-y-6">
                  <div>
                    <h3 className="text-lg font-medium text-gray-900 mb-3">Recommendations</h3>
                    <div className="space-y-2">
                      {security_score.recommendations.map((recommendation, index) => (
                        <div key={index} className="flex items-start space-x-3 p-3 bg-blue-50 rounded-lg">
                          <Info className="h-5 w-5 text-blue-500 mt-0.5" />
                          <div className="text-sm text-blue-800">{recommendation}</div>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              )}

              {activeTab === 'reused' && (
                <div className="space-y-4">
                  <h3 className="text-lg font-medium text-gray-900">Reused Password Groups</h3>
                  {dashboardData.reused_groups.length === 0 ? (
                    <div className="text-center py-8 text-gray-500">
                      <CheckCircle className="h-12 w-12 text-green-500 mx-auto mb-2" />
                      <div>No reused passwords found - excellent!</div>
                    </div>
                  ) : (
                    <div className="space-y-3">
                      {dashboardData.reused_groups.map((group, index) => (
                        <div key={index} className="flex items-center justify-between p-4 bg-gray-50 rounded-lg">
                          <div className="flex items-center space-x-3">
                            {getRiskIcon(group.risk_level)}
                            <div>
                              <div className="font-medium">Group #{index + 1}</div>
                              <div className="text-sm text-gray-600">
                                {group.entries.length} accounts using the same password
                              </div>
                            </div>
                          </div>
                          <div className={`px-2 py-1 rounded text-xs font-medium ${
                            group.risk_level === 'Critical' ? 'bg-red-100 text-red-800' :
                            group.risk_level === 'High' ? 'bg-orange-100 text-orange-800' :
                            group.risk_level === 'Medium' ? 'bg-yellow-100 text-yellow-800' :
                            'bg-green-100 text-green-800'
                          }`}>
                            {group.risk_level} Risk
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              )}

              {activeTab === 'age' && (
                <div className="space-y-4">
                  <h3 className="text-lg font-medium text-gray-900">Password Age Distribution</h3>
                  {renderAgeChart(dashboardData.age_distribution)}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}