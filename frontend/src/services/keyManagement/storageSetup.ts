import type { DBSchema } from 'idb';

interface MyDatabase extends DBSchema {
  keys: {
    key: number;
    value: {
      pub: JsonWebKey;
      priv: {
        jwe: string;
        salt: number[];
      };
      kid: number;
    };
  };
}

// Simple IndexedDB wrapper to replace @adorsys-gis/storage
class SimpleStorage {
  private dbName: string;
  private version: number;
  private db: IDBDatabase | null = null;
  private isReady = false;

  constructor(dbName: string, version: number) {
    this.dbName = dbName;
    this.version = version;
  }

  async init(): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(this.dbName, this.version);

      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        this.db = request.result;
        this.isReady = true;
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;
        if (!db.objectStoreNames.contains('keys')) {
          db.createObjectStore('keys', {
            keyPath: 'kid',
            autoIncrement: true,
          });
        }
        if (!db.objectStoreNames.contains('metadata')) {
          db.createObjectStore('metadata', {
            keyPath: 'key',
          });
        }
      };
    });
  }

  async waitForReady(): Promise<void> {
    if (this.isReady) return;

    // Wait for initialization to complete
    let attempts = 0;
    while (!this.isReady && attempts < 50) {
      await new Promise((resolve) => setTimeout(resolve, 100));
      attempts++;
    }

    if (!this.isReady) {
      throw new Error('Storage initialization timeout');
    }
  }

  async insert(storeName: string, data: Record<string, unknown>): Promise<void> {
    await this.waitForReady();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([storeName], 'readwrite');
      const store = transaction.objectStore(storeName);
      const request = store.add(data);

      request.onerror = () => reject(request.error);
      request.onsuccess = () => resolve();
    });
  }

  async findOne(storeName: string, key: number): Promise<Record<string, unknown> | undefined> {
    await this.waitForReady();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([storeName], 'readonly');
      const store = transaction.objectStore(storeName);
      const request = store.get(key);

      request.onerror = () => reject(request.error);
      request.onsuccess = () => resolve(request.result);
    });
  }

  async clear(storeName: string): Promise<void> {
    await this.waitForReady();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([storeName], 'readwrite');
      const store = transaction.objectStore(storeName);
      const request = store.clear();

      request.onerror = () => reject(request.error);
      request.onsuccess = () => resolve();
    });
  }

  async count(storeName: string): Promise<number> {
    await this.waitForReady();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([storeName], 'readonly');
      const store = transaction.objectStore(storeName);
      const request = store.count();

      request.onerror = () => reject(request.error);
      request.onsuccess = () => resolve(request.result);
    });
  }
}

// Initialize the storage
const storage = new SimpleStorage('EventKeyStorage', 2);

// Initialize storage on module load
storage.init().catch(() => {
  // Silently handle storage initialization error
});

// Add a function to clear all stored data
export async function clearAllStoredData() {
  await storage.clear('keys');
  // Clear other storage if needed
  localStorage.removeItem('messages');
  sessionStorage.removeItem('password');
}

export default storage;
