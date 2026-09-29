import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import OpenWorkerPage from './OpenWorkerPage';

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

// Mock coreRpcClient
const mockCallCoreRpc = vi.fn();
vi.mock('../../services/coreRpcClient', () => ({
  callCoreRpc: (req: { method: string; params?: unknown }) => mockCallCoreRpc(req),
}));

describe('OpenWorkerPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders header, offline status, and start button when offline', async () => {
    mockCallCoreRpc.mockImplementation(async ({ method }) => {
      if (method === 'openworker.status') {
        return { running: false };
      }
      return null;
    });

    render(<OpenWorkerPage />);

    expect(screen.getByText(/OpenWorker Specialists/i)).toBeInTheDocument();
    expect(screen.getByText(/Server offline/i)).toBeInTheDocument();
    expect(screen.getByText(/Start server/i)).toBeInTheDocument();
    expect(screen.getByText(/Start the server to load specialists/i)).toBeInTheDocument();
  });

  it('displays running status and renders coworker cards when online', async () => {
    mockCallCoreRpc.mockImplementation(async ({ method }) => {
      if (method === 'openworker.status') {
        return { running: true, port: 8899 };
      }
      if (method === 'openworker.list_coworkers') {
        return [
          {
            id: 'security',
            name: 'Security Auditor',
            description: 'Custom security scanner',
            capabilities: ['sast'],
          },
          {
            id: 'cloud',
            name: 'Cloud Architect',
            description: 'Cloud infrastructure auditor',
            capabilities: ['terraform', 'aws'],
          },
        ];
      }
      return null;
    });

    render(<OpenWorkerPage />);

    await waitFor(() => {
      expect(screen.getByText(/Server running/i)).toBeInTheDocument();
    });
    expect(screen.getByText('Stop')).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByText('Security Auditor')).toBeInTheDocument();
      expect(screen.getByText('Cloud Architect')).toBeInTheDocument();
    });
  });

  it('allows selecting a coworker and delegating a goal', async () => {
    mockCallCoreRpc.mockImplementation(async ({ method }) => {
      if (method === 'openworker.status') {
        return { running: true, port: 8899 };
      }
      if (method === 'openworker.list_coworkers') {
        return [
          {
            id: 'cloud',
            name: 'Cloud Architect',
            description: 'Cloud infrastructure auditor',
            capabilities: ['terraform', 'aws'],
          },
        ];
      }
      if (method === 'openworker.delegate') {
        return 'job-test-789';
      }
      return null;
    });

    render(<OpenWorkerPage />);

    // Wait for coworker to load
    await waitFor(() => {
      expect(screen.getByText('Cloud Architect')).toBeInTheDocument();
    });

    // Click on Cloud Architect
    fireEvent.click(screen.getByText('Cloud Architect'));

    // Type goal
    const textarea = screen.getByPlaceholderText(/Describe the outcome you want/i);
    fireEvent.change(textarea, { target: { value: 'Audit AWS S3 buckets for public access' } });

    // Click delegate
    const delegateBtn = screen.getByText(/Delegate task/i);
    fireEvent.click(delegateBtn);

    await waitFor(() => {
      expect(mockCallCoreRpc).toHaveBeenCalledWith(
        expect.objectContaining({
          method: 'openworker.delegate',
          params: expect.objectContaining({
            coworker: 'cloud',
            goal: 'Audit AWS S3 buckets for public access',
          }),
        })
      );
    });
  });
});
