export class PasswordManager {
  static async getPassword(): Promise<string> {
    // Check sessionStorage for the password first
    const sessionPassword = sessionStorage.getItem('password');
    if (sessionPassword) {
      return sessionPassword;
    }

    // If not in session, check localStorage
    const localPassword = localStorage.getItem('password');
    if (localPassword) {
      // Store it in sessionStorage for the current session
      sessionStorage.setItem('password', localPassword);
      return localPassword;
    }

    // If no password exists, generate a new one
    const newPassword = this.generateSecurePassword();
    localStorage.setItem('password', newPassword);
    sessionStorage.setItem('password', newPassword);
    return newPassword;
  }

  private static generateSecurePassword(): string {
    const array = new Uint8Array(32);
    window.crypto.getRandomValues(array);
    return btoa(String.fromCharCode(...array)).slice(0, 32);
  }
}
