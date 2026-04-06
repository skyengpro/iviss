import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import storage, { clearAllStoredData } from '../storageSetup';

const dummyDB = {
  close: vi.fn(),
  transaction: vi.fn(),
  objectStoreNames: {
    contains: vi.fn(),
  },
  createObjectStore: vi.fn(),
  onversionchange: null as any,
  onabort: null as any,
  onerror: null as any,
};

const mockIDBOpenDBRequest = {
  result: dummyDB,
  error: null as any,
  onupgradeneeded: null as any,
  onsuccess: null as any,
  onerror: null as any,
  onblocked: null as any,
};

// Helper macro to flush microtasks
const flushPromises = () => new Promise(setImmediate);

describe('SimpleStorage', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    vi.stubGlobal('indexedDB', {
      open: vi.fn(() => mockIDBOpenDBRequest),
    });

    (storage as any).initPromise = null;
    (storage as any).db = null;
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  const triggerOpenSuccess = () => {
    if (mockIDBOpenDBRequest.onsuccess) {
      (mockIDBOpenDBRequest.onsuccess as Function)();
    }
  };

  it('init() initializes DB and creates object stores', async () => {
    dummyDB.objectStoreNames.contains.mockReturnValue(false);

    const initPromise = storage.init();

    expect(indexedDB.open).toHaveBeenCalledWith('EventKeyStorage', 2);

    if (mockIDBOpenDBRequest.onupgradeneeded) {
      (mockIDBOpenDBRequest.onupgradeneeded as Function)({
        target: { result: dummyDB },
      });
    }

    expect(dummyDB.createObjectStore).toHaveBeenCalledWith('keys', {
      keyPath: 'kid',
      autoIncrement: true,
    });
    expect(dummyDB.createObjectStore).toHaveBeenCalledWith('metadata', { keyPath: 'key' });

    triggerOpenSuccess();
    await initPromise;

    expect((storage as any).db).toBe(dummyDB);
  });

  const setupMockTransaction = (storeMethods: Record<string, any>) => {
    const mockStore = { ...storeMethods };
    const mockTransaction = { objectStore: vi.fn(() => mockStore) };
    dummyDB.transaction.mockReturnValue(mockTransaction);
    return mockStore;
  };

  it('insert() calls store.add with correct data', async () => {
    const initPromise = storage.init();
    triggerOpenSuccess();
    await initPromise;

    const mockRequest = { onsuccess: null as any, onerror: null as any };
    const mockStore = setupMockTransaction({
      add: vi.fn(() => mockRequest),
    });

    const insertData = { id: 1, val: 'test' };
    const insertPromise = storage.insert('keys', insertData);

    // Wait until `await this.getDB()` finishes and block enters
    await flushPromises();

    mockRequest.onsuccess(); // Now this should be populated
    await insertPromise;

    expect(dummyDB.transaction).toHaveBeenCalledWith(['keys'], 'readwrite');
    expect(mockStore.add).toHaveBeenCalledWith(insertData);
  });

  it('findOne() calls store.get with correct key', async () => {
    const initPromise = storage.init();
    triggerOpenSuccess();
    await initPromise;

    const mockRequest = {
      result: { kid: 42, data: 'test-data' },
      onsuccess: null as any,
      onerror: null as any,
    };
    const mockStore = setupMockTransaction({
      get: vi.fn(() => mockRequest),
    });

    const findPromise = storage.findOne('keys', 42);

    await flushPromises();

    mockRequest.onsuccess();
    const result = await findPromise;

    expect(dummyDB.transaction).toHaveBeenCalledWith(['keys'], 'readonly');
    expect(mockStore.get).toHaveBeenCalledWith(42);
    expect(result).toEqual({ kid: 42, data: 'test-data' });
  });

  it('clear() calls store.clear', async () => {
    const initPromise = storage.init();
    triggerOpenSuccess();
    await initPromise;

    const mockRequest = { onsuccess: null as any, onerror: null as any };
    const mockStore = setupMockTransaction({
      clear: vi.fn(() => mockRequest),
    });

    const clearPromise = storage.clear('keys');

    await flushPromises();

    mockRequest.onsuccess();
    await clearPromise;

    expect(dummyDB.transaction).toHaveBeenCalledWith(['keys'], 'readwrite');
    expect(mockStore.clear).toHaveBeenCalled();
  });

  it('count() returns the store record count', async () => {
    const initPromise = storage.init();
    triggerOpenSuccess();
    await initPromise;

    const mockRequest = { result: 5, onsuccess: null as any, onerror: null as any };
    const mockStore = setupMockTransaction({
      count: vi.fn(() => mockRequest),
    });

    const countPromise = storage.count('keys');

    await flushPromises();

    mockRequest.onsuccess();
    const count = await countPromise;

    expect(dummyDB.transaction).toHaveBeenCalledWith(['keys'], 'readonly');
    expect(mockStore.count).toHaveBeenCalled();
    expect(count).toBe(5);
  });

  it('clearAllStoredData clears both keys and metadata stores', async () => {
    const initPromise = storage.init();
    triggerOpenSuccess();
    await initPromise;

    vi.spyOn(storage, 'clear')
      .mockResolvedValueOnce(undefined) // keys
      .mockResolvedValueOnce(undefined); // metadata

    await clearAllStoredData();

    expect(storage.clear).toHaveBeenCalledWith('keys');
    expect(storage.clear).toHaveBeenCalledWith('metadata');
  });

  it('retries on InvalidStateError', async () => {
    const initPromise = storage.init();
    triggerOpenSuccess();
    await initPromise;

    const error = new Error('Invalid state');
    error.name = 'InvalidStateError';

    dummyDB.transaction.mockImplementationOnce(() => {
      throw error;
    });

    const mockRequest = { result: 1, onsuccess: null as any, onerror: null as any };
    setupMockTransaction({
      count: vi.fn(() => mockRequest),
    });

    const countPromise = storage.count('keys');

    // Wait for initial failure and retry (which calls this.getDB() -> this.init())
    await flushPromises();
    // Simulate open DB success again
    triggerOpenSuccess();
    // Wait for the setup to complete
    await flushPromises();

    mockRequest.onsuccess();
    const count = await countPromise;

    expect(dummyDB.transaction).toHaveBeenCalledTimes(2);
    expect(count).toBe(1);
  });
});
