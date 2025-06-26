import { create } from 'zustand'
import { TodoState, TodoItem } from '../types/todo' // Updated import
import { persist, createJSONStorage } from 'zustand/middleware'

export interface TodoStore extends TodoState {
  setTasks: (tasks: TodoItem[]) => void; // Renamed action
  get: () => TodoStore;
  set: (partial: TodoStore | Partial<TodoStore>) => void;
}

// Kept store hook name the same for simplicity, but could be renamed e.g. useTodoStore
const useTodoStore = create<TodoStore>()( 
  persist(
    (set, get) => ({
      tasks: [], // Initialize state with an empty array of TodoItems
      setTasks: (newTasks: TodoItem[]) => { // Renamed action implementation
        set({ tasks: newTasks });
      },
      get,
      set,
    }),
    {
      name: 'todo-store', // Changed persistence key for clarity
      storage: createJSONStorage(() => sessionStorage), 
    }
  )
)

export default useTodoStore; 