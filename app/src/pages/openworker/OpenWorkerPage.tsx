import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { callCoreRpc } from '../../services/coreRpcClient';

// ── Types ─────────────────────────────────────────────────────────────────────

type CoworkerStatus = 'idle' | 'running' | 'awaiting_approval' | 'completed' | 'failed';

interface Coworker {
  id: string;
  name: string;
  description: string;
  capabilities: string[];
}

interface ApprovalGate {
  action_id: string;
  description: string;
  payload?: unknown;
}

interface Job {
  job_id: string;
  coworker: string;
  goal: string;
  status: CoworkerStatus;
  progress: string;
  result?: string;
  error?: string;
  approvals_needed: ApprovalGate[];
}

// ── RPC helpers ───────────────────────────────────────────────────────────────

async function rpc<T>(method: string, params: Record<string, unknown> = {}): Promise<T> {
  return callCoreRpc<T>({ method, params });
}

// ── Sub-components ────────────────────────────────────────────────────────────

function StatusDot({ running }: { running: boolean }) {
  return (
    <span
      className="inline-block w-2 h-2 rounded-full"
      style={{ background: running ? '#22c55e' : '#71717a' }}
    />
  );
}

function CoworkerBadge({ id }: { id: string }) {
  const colors: Record<string, string> = {
    security: '#ef4444',
    cloud: '#3b82f6',
    document: '#8b5cf6',
    slack: '#f59e0b',
    calendar: '#10b981',
    automation: '#6366f1',
  };
  return (
    <span
      className="text-xs font-semibold px-2 py-0.5 rounded-full text-white"
      style={{ background: colors[id] ?? '#6b7280' }}
    >
      {id}
    </span>
  );
}

function ApprovalCard({
  job,
  gate,
  onRespond,
}: {
  job: Job;
  gate: ApprovalGate;
  onRespond: (jobId: string, actionId: string, approved: boolean) => void;
}) {
  return (
    <div className="border border-yellow-400/40 bg-yellow-500/5 rounded-lg p-3 space-y-2">
      <p className="text-sm font-medium text-yellow-300">⚠ Approval Required</p>
      <p className="text-sm text-content-secondary">{gate.description}</p>
      <div className="flex gap-2">
        <button
          id={`approve-${gate.action_id}`}
          className="px-3 py-1 text-xs rounded-md bg-green-600 hover:bg-green-500 text-white font-medium transition-colors"
          onClick={() => onRespond(job.job_id, gate.action_id, true)}
        >
          Approve
        </button>
        <button
          id={`deny-${gate.action_id}`}
          className="px-3 py-1 text-xs rounded-md bg-red-600 hover:bg-red-500 text-white font-medium transition-colors"
          onClick={() => onRespond(job.job_id, gate.action_id, false)}
        >
          Deny
        </button>
      </div>
    </div>
  );
}

function JobCard({
  job,
  onApprove,
  onCancel,
}: {
  job: Job;
  onApprove: (jobId: string, actionId: string, approved: boolean) => void;
  onCancel: (jobId: string) => void;
}) {
  const statusColors: Record<CoworkerStatus, string> = {
    idle: 'text-content-muted',
    running: 'text-blue-400',
    awaiting_approval: 'text-yellow-400',
    completed: 'text-green-400',
    failed: 'text-red-400',
  };

  return (
    <div className="rounded-xl border border-surface-strong bg-surface p-4 space-y-3 shadow-sm">
      <div className="flex items-start justify-between gap-2">
        <div className="space-y-1 min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <CoworkerBadge id={job.coworker} />
            <span className={`text-xs font-medium ${statusColors[job.status]}`}>
              {job.status.replace('_', ' ')}
            </span>
          </div>
          <p className="text-sm font-medium text-content truncate">{job.goal}</p>
          <p className="text-xs text-content-muted font-mono">{job.job_id}</p>
        </div>
        {job.status === 'running' || job.status === 'awaiting_approval' ? (
          <button
            id={`cancel-job-${job.job_id}`}
            className="text-xs text-red-400 hover:text-red-300 shrink-0 transition-colors"
            onClick={() => onCancel(job.job_id)}
          >
            Cancel
          </button>
        ) : null}
      </div>

      {job.progress && (
        <p className="text-xs text-content-secondary">{job.progress}</p>
      )}

      {job.approvals_needed.map(gate => (
        <ApprovalCard key={gate.action_id} job={job} gate={gate} onRespond={onApprove} />
      ))}

      {job.result && (
        <div className="bg-surface-muted rounded-lg p-3">
          <p className="text-xs text-content-muted mb-1 font-semibold uppercase tracking-wider">
            Result
          </p>
          <pre className="text-xs text-content whitespace-pre-wrap break-words">{job.result}</pre>
        </div>
      )}

      {job.error && (
        <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-3">
          <p className="text-xs text-red-400">{job.error}</p>
        </div>
      )}
    </div>
  );
}

// ── Main page ─────────────────────────────────────────────────────────────────

export default function OpenWorkerPage() {
  const [serverRunning, setServerRunning] = useState(false);
  const [starting, setStarting] = useState(false);
  const [coworkers, setCoworkers] = useState<Coworker[]>([]);
  const [jobs, setJobs] = useState<Job[]>([]);
  const [selectedCoworker, setSelectedCoworker] = useState('');
  const [goal, setGoal] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // ── Server status ───────────────────────────────────────────────────────────

  const checkStatus = useCallback(async () => {
    try {
      const status = await rpc<{ running: boolean }>('openworker.status');
      setServerRunning(status.running);
      if (status.running && coworkers.length === 0) {
        const cws = await rpc<Coworker[]>('openworker.list_coworkers');
        setCoworkers(cws);
      }
    } catch {
      setServerRunning(false);
    }
  }, [coworkers.length]);

  useEffect(() => {
    checkStatus();
    const id = setInterval(checkStatus, 5000);
    return () => clearInterval(id);
  }, [checkStatus]);

  // ── Job polling ─────────────────────────────────────────────────────────────

  const pollJobs = useCallback(async () => {
    const active = jobs.filter(
      j => j.status === 'running' || j.status === 'awaiting_approval',
    );
    if (active.length === 0) return;

    const updated = await Promise.all(
      active.map(j => rpc<Job>('openworker.job_state', { job_id: j.job_id }).catch(() => j)),
    );
    setJobs(prev =>
      prev.map(j => updated.find(u => u.job_id === j.job_id) ?? j),
    );
  }, [jobs]);

  useEffect(() => {
    pollRef.current = setInterval(pollJobs, 2500);
    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
    };
  }, [pollJobs]);

  // ── Actions ─────────────────────────────────────────────────────────────────

  const startServer = async () => {
    setStarting(true);
    try {
      await invoke('openworker_start');
      await checkStatus();
    } catch (e) {
      console.error('[openworker] start failed:', e);
    } finally {
      setStarting(false);
    }
  };

  const stopServer = async () => {
    await invoke('openworker_stop');
    setServerRunning(false);
    setCoworkers([]);
  };

  const submitJob = async () => {
    if (!selectedCoworker || !goal.trim()) return;
    setSubmitting(true);
    try {
      const jobId = await rpc<string>('openworker.delegate', {
        coworker: selectedCoworker,
        goal: goal.trim(),
      });
      const newJob: Job = {
        job_id: jobId,
        coworker: selectedCoworker,
        goal: goal.trim(),
        status: 'running',
        progress: 'Starting…',
        approvals_needed: [],
      };
      setJobs(prev => [newJob, ...prev]);
      setGoal('');
    } catch (e) {
      console.error('[openworker] delegate failed:', e);
    } finally {
      setSubmitting(false);
    }
  };

  const handleApprove = async (jobId: string, actionId: string, approved: boolean) => {
    await rpc('openworker.approve', { job_id: jobId, action_id: actionId, approved });
  };

  const handleCancel = async (jobId: string) => {
    await rpc('openworker.cancel', { job_id: jobId });
    setJobs(prev =>
      prev.map(j => (j.job_id === jobId ? { ...j, status: 'failed' as CoworkerStatus, error: 'Cancelled by user.' } : j)),
    );
  };

  // ── Render ──────────────────────────────────────────────────────────────────

  return (
    <div className="flex flex-col h-full bg-surface-canvas">
      {/* Header */}
      <div className="px-6 py-5 border-b border-surface-strong flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-content flex items-center gap-2">
            <span>⚡</span> OpenWorker Specialists
          </h1>
          <p className="text-sm text-content-muted mt-0.5">
            Delegate deep specialist tasks to governed AI coworkers.
          </p>
        </div>
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1.5 text-sm text-content-secondary">
            <StatusDot running={serverRunning} />
            {serverRunning ? 'Server running' : 'Server offline'}
          </div>
          {serverRunning ? (
            <button
              id="openworker-stop-btn"
              onClick={stopServer}
              className="px-3 py-1.5 text-xs rounded-lg bg-surface-strong hover:bg-surface-hover text-content-secondary transition-colors"
            >
              Stop
            </button>
          ) : (
            <button
              id="openworker-start-btn"
              onClick={startServer}
              disabled={starting}
              className="px-3 py-1.5 text-xs rounded-lg bg-primary-500 hover:bg-primary-400 text-white font-medium transition-colors disabled:opacity-50"
            >
              {starting ? 'Starting…' : 'Start server'}
            </button>
          )}
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar — coworker selector + dispatch */}
        <div className="w-72 shrink-0 border-r border-surface-strong p-4 flex flex-col gap-4 overflow-y-auto">
          <div className="space-y-1">
            <label className="text-xs font-semibold text-content-muted uppercase tracking-wider">
              Specialist
            </label>
            {coworkers.length > 0 ? (
              <div className="space-y-1">
                {coworkers.map(cw => (
                  <button
                    key={cw.id}
                    id={`coworker-select-${cw.id}`}
                    onClick={() => setSelectedCoworker(cw.id)}
                    className={`w-full text-left rounded-lg px-3 py-2.5 transition-colors ${
                      selectedCoworker === cw.id
                        ? 'bg-primary-500/10 border border-primary-500/30'
                        : 'hover:bg-surface-hover border border-transparent'
                    }`}
                  >
                    <div className="flex items-center gap-2 mb-0.5">
                      <CoworkerBadge id={cw.id} />
                      <span className="text-sm font-medium text-content">{cw.name}</span>
                    </div>
                    <p className="text-xs text-content-muted mt-1">{cw.description}</p>
                  </button>
                ))}
              </div>
            ) : (
              <p className="text-xs text-content-faint italic">
                {serverRunning ? 'Loading coworkers…' : 'Start the server to load specialists.'}
              </p>
            )}
          </div>

          {/* Goal input + submit */}
          <div className="space-y-2 mt-auto">
            <label className="text-xs font-semibold text-content-muted uppercase tracking-wider">
              Goal
            </label>
            <textarea
              id="openworker-goal-input"
              value={goal}
              onChange={e => setGoal(e.target.value)}
              placeholder="Describe the outcome you want…"
              rows={4}
              className="w-full rounded-lg border border-surface-strong bg-surface-muted p-3 text-sm text-content placeholder:text-content-faint resize-none focus:outline-none focus:border-primary-500/50 transition-colors"
            />
            <button
              id="openworker-submit-btn"
              onClick={submitJob}
              disabled={!serverRunning || !selectedCoworker || !goal.trim() || submitting}
              className="w-full py-2 rounded-lg bg-primary-500 hover:bg-primary-400 text-white text-sm font-medium transition-colors disabled:opacity-40"
            >
              {submitting ? 'Delegating…' : 'Delegate task'}
            </button>
          </div>
        </div>

        {/* Job list */}
        <div className="flex-1 overflow-y-auto p-4 space-y-3">
          {jobs.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-center gap-3 text-content-faint">
              <span className="text-4xl">🤖</span>
              <p className="text-sm">No jobs yet. Select a specialist and describe a goal.</p>
            </div>
          ) : (
            jobs.map(job => (
              <JobCard
                key={job.job_id}
                job={job}
                onApprove={handleApprove}
                onCancel={handleCancel}
              />
            ))
          )}
        </div>
      </div>
    </div>
  );
}
