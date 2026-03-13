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
  private initPromise: Promise<void> | null = null;

  constructor(dbName: string, version: number) {
    this.dbName = dbName;
    this.version = version;
  }

  async init(): Promise<void> {
    if (this.initPromise) return this.initPromise;

    this.initPromise = (async () => {
      return new Promise((resolve, reject) => {
        const request = indexedDB.open(this.dbName, this.version);

        request.onerror = () => {
          console.error(`IDB open error (${this.dbName}):`, request.error);
          this.initPromise = null;
          reject(request.error);
        };

        request.onblocked = () => {
          console.warn(`IDB open blocked (${this.dbName}). Close other tabs!`);
        };

        request.onsuccess = () => {
          this.db = request.result;
          this.db.onversionchange = () => {
            console.warn(`IDB version change (${this.dbName}), closing connection...`);
            this.db?.close();
            this.db = null;
          };
          this.db.onabort = () => {
            console.warn(`IDB connection aborted (${this.dbName})`);
            this.db = null;
          };
          this.db.onerror = (err) => {
            console.error(`IDB global error (${this.dbName}):`, err);
          };
          resolve();
        };

        request.onupgradeneeded = (event) => {
          const db = (event.target as IDBOpenDBRequest).result;
          if (!db.objectStoreNames.contains('keys')) {
            db.createObjectStore('keys', { keyPath: 'kid', autoIncrement: true });
          }
          if (!db.objectStoreNames.contains('metadata')) {
            db.createObjectStore('metadata', { keyPath: 'key' });
          }
        };
      });
    })();

    return this.initPromise;
  }

  private async getDB(): Promise<IDBDatabase> {
    await this.init();

    // Check if connection is closed or closing
    // Note: there is no direct "isClosed" property, but we can detect it by trying a transaction
    // or checking our null state.
    if (!this.db) {
      this.initPromise = null; // force re-init
      await this.init();
    }

    if (!this.db) {
      throw new Error('Failed to initialize IndexedDB');
    }

    return this.db;
  }

  async insert(storeName: string, data: Record<string, unknown>): Promise<void> {
    const db = await this.getDB();

    return new Promise((resolve, reject) => {
      try {
        const transaction = db.transaction([storeName], 'readwrite');
        const store = transaction.objectStore(storeName);
        const request = store.add(data);

        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve();
      } catch (err) {
        if (err instanceof Error && err.name === 'InvalidStateError') {
          // Connection likely closing, force refresh and retry once
          this.db = null;
          this.getDB().then(() => this.insert(storeName, data)).then(resolve).catch(reject);
        } else {
          reject(err);
        }
      }
    });
  }

  async findOne(
    storeName: string,
    key: string | number
  ): Promise<Record<string, unknown> | undefined> {
    const db = await this.getDB();

    return new Promise((resolve, reject) => {
      try {
        const transaction = db.transaction([storeName], 'readonly');
        const store = transaction.objectStore(storeName);
        const request = store.get(key);

        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve(request.result);
      } catch (err) {
        if (err instanceof Error && err.name === 'InvalidStateError') {
          this.db = null;
          this.getDB()
            .then(() => this.findOne(storeName, key))
            .then(resolve as (value: Record<string, unknown> | undefined) => void)
            .catch(reject);
        } else {
          reject(err);
        }
      }
    });
  }

  async clear(storeName: string): Promise<void> {
    const db = await this.getDB();

    return new Promise((resolve, reject) => {
      try {
        const transaction = db.transaction([storeName], 'readwrite');
        const store = transaction.objectStore(storeName);
        const request = store.clear();

        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve();
      } catch (err) {
        if (err instanceof Error && err.name === 'InvalidStateError') {
          this.db = null;
          this.getDB().then(() => this.clear(storeName)).then(resolve).catch(reject);
        } else {
          reject(err);
        }
      }
    });
  }

  async count(storeName: string): Promise<number> {
    const db = await this.getDB();

    return new Promise((resolve, reject) => {
      try {
        const transaction = db.transaction([storeName], 'readonly');
        const store = transaction.objectStore(storeName);
        const request = store.count();

        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve(request.result);
      } catch (err) {
        if (err instanceof Error && err.name === 'InvalidStateError') {
          this.db = null;
          this.getDB().then(() => this.count(storeName)).then(resolve).catch(reject);
        } else {
          reject(err);
        }
      }
    });
  }
}

// Initialize the storage
const storage = new SimpleStorage('EventKeyStorage', 2);

// Initialize storage on module load
storage.init().catch(() => { });

// Add a function to clear all stored data
export async function clearAllStoredData() {
  try {
    await storage.clear('keys');
    await storage.clear('metadata');
  } catch (err) {
    console.warn('Silent cleanup of IndexedDB stores failed (this is usually okay):', err);
  }
}

export default storage;
