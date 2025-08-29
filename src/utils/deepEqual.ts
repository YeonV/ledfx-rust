export const deepEqual = (obj1: any, obj2: any): boolean => {
    if (!obj1 || !obj2 || typeof obj1 !== 'object' || typeof obj2 !== 'object') {
        return obj1 === obj2;
    }

    const keys1 = Object.keys(obj1);
    const keys2 = Object.keys(obj2);

    // The preset might have more keys than the current settings if the user hasn't touched them yet.
    // We only need to check if all keys in the *preset* match the current settings.
    for (const key of keys2) {
        if (!keys1.includes(key) || !deepEqual(obj1[key], obj2[key])) {
            return false;
        }
    }
    
    // Also check if the user settings have extra keys not in the preset.
    // This handles the case where a schema might change.
    for (const key of keys1) {
        if(!keys2.includes(key)) {
            return false;
        }
    }
    
    return true;
}
