export * from '../bindings.ts'; 

// You can also add custom logic here in the future.
// For example, adding logging to every command call.
// export const loggedCommands = new Proxy(commands, {
//     get(target, prop, receiver) {
//         const original = target[prop];
//         if (typeof original === 'function') {
//             return async function(...args) {
//                 console.log(`Calling command: ${String(prop)} with args:`, args);
//                 return original.apply(this, args);
//             }
//         }
//         return original;
//     }
// });