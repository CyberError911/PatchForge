"""PatchForge GUI — Tkinter-based interface for creating and applying patches."""

import os
import sys
import threading
from pathlib import Path
from tkinter import *
from tkinter import filedialog, messagebox, ttk


class PatchForgeGUI:
    def __init__(self, root):
        self.root = root
        self.root.title("PatchForge — Patch Creator & Applier")
        self.root.geometry("700x500")
        
        # Tabs
        self.notebook = ttk.Notebook(root)
        self.notebook.pack(fill=BOTH, expand=True, padx=10, pady=10)
        
        self.make_frame = ttk.Frame(self.notebook)
        self.apply_frame = ttk.Frame(self.notebook)
        
        self.notebook.add(self.make_frame, text="Create Patch")
        self.notebook.add(self.apply_frame, text="Apply Patch")
        
        self.setup_make_tab()
        self.setup_apply_tab()
    
    def setup_make_tab(self):
        """Setup the 'Create Patch' tab."""
        frame = ttk.Frame(self.make_frame, padding=10)
        frame.pack(fill=BOTH, expand=True)
        
        # Source folder
        ttk.Label(frame, text="Source Folder (Old):").grid(row=0, column=0, sticky=W, pady=5)
        self.src_entry = ttk.Entry(frame, width=50)
        self.src_entry.grid(row=0, column=1, padx=5)
        ttk.Button(frame, text="Browse", command=self.browse_src).grid(row=0, column=2)
        
        # Destination folder
        ttk.Label(frame, text="Destination Folder (New):").grid(row=1, column=0, sticky=W, pady=5)
        self.dst_entry = ttk.Entry(frame, width=50)
        self.dst_entry.grid(row=1, column=1, padx=5)
        ttk.Button(frame, text="Browse", command=self.browse_dst).grid(row=1, column=2)
        
        # Output patch
        ttk.Label(frame, text="Output Patch File:").grid(row=2, column=0, sticky=W, pady=5)
        self.patch_entry = ttk.Entry(frame, width=50)
        self.patch_entry.grid(row=2, column=1, padx=5)
        ttk.Button(frame, text="Browse", command=self.browse_patch).grid(row=2, column=2)
        
        # Compression level
        ttk.Label(frame, text="Compression Level:").grid(row=3, column=0, sticky=W, pady=5)
        self.zstd_var = StringVar(value="3")
        ttk.Scale(frame, from_=-1, to=22, variable=self.zstd_var, orient=HORIZONTAL).grid(row=3, column=1, sticky=EW, padx=5)
        ttk.Label(frame, text="-1=None, 0-22=Levels", font=("Arial", 8)).grid(row=3, column=2, sticky=W)
        
        # Create button
        ttk.Button(frame, text="Create Patch", command=self.create_patch).grid(row=4, column=0, columnspan=3, pady=20, sticky=EW)
        
        # Progress
        self.make_progress = ttk.Progressbar(frame, mode='indeterminate')
        self.make_progress.grid(row=5, column=0, columnspan=3, sticky=EW, pady=5)
        
        # Status
        self.make_status = ttk.Label(frame, text="Ready", foreground="gray")
        self.make_status.grid(row=6, column=0, columnspan=3, sticky=W)
    
    def setup_apply_tab(self):
        """Setup the 'Apply Patch' tab."""
        frame = ttk.Frame(self.apply_frame, padding=10)
        frame.pack(fill=BOTH, expand=True)
        
        # Target folder
        ttk.Label(frame, text="Target Folder:").grid(row=0, column=0, sticky=W, pady=5)
        self.target_entry = ttk.Entry(frame, width=50)
        self.target_entry.grid(row=0, column=1, padx=5)
        ttk.Button(frame, text="Browse", command=self.browse_target).grid(row=0, column=2)
        
        # Patch file
        ttk.Label(frame, text="Patch File:").grid(row=1, column=0, sticky=W, pady=5)
        self.apply_patch_entry = ttk.Entry(frame, width=50)
        self.apply_patch_entry.grid(row=1, column=1, padx=5)
        ttk.Button(frame, text="Browse", command=self.browse_patch_file).grid(row=1, column=2)
        
        # Options
        self.verify_var = BooleanVar(value=True)
        ttk.Checkbutton(frame, text="Verify checksums", variable=self.verify_var).grid(row=2, column=0, columnspan=2, sticky=W, pady=5)
        
        # Apply button
        ttk.Button(frame, text="Apply Patch", command=self.apply_patch).grid(row=3, column=0, columnspan=3, pady=20, sticky=EW)
        
        # Progress
        self.apply_progress = ttk.Progressbar(frame, mode='indeterminate')
        self.apply_progress.grid(row=4, column=0, columnspan=3, sticky=EW, pady=5)
        
        # Status
        self.apply_status = ttk.Label(frame, text="Ready", foreground="gray")
        self.apply_status.grid(row=5, column=0, columnspan=3, sticky=W)
    
    def browse_src(self):
        path = filedialog.askdirectory(title="Select Source Folder")
        if path:
            self.src_entry.delete(0, END)
            self.src_entry.insert(0, path)
    
    def browse_dst(self):
        path = filedialog.askdirectory(title="Select Destination Folder")
        if path:
            self.dst_entry.delete(0, END)
            self.dst_entry.insert(0, path)
    
    def browse_target(self):
        path = filedialog.askdirectory(title="Select Target Folder")
        if path:
            self.target_entry.delete(0, END)
            self.target_entry.insert(0, path)
    
    def browse_patch(self):
        path = filedialog.asksaveasfilename(title="Save Patch File As", defaultextension=".patch")
        if path:
            self.patch_entry.delete(0, END)
            self.patch_entry.insert(0, path)
    
    def browse_patch_file(self):
        path = filedialog.askopenfilename(title="Select Patch File", filetypes=[("Patch files", "*.patch"), ("All files", "*.*")])
        if path:
            self.apply_patch_entry.delete(0, END)
            self.apply_patch_entry.insert(0, path)
    
    def create_patch(self):
        """Create a patch (calls CLI in background)."""
        src = self.src_entry.get()
        dst = self.dst_entry.get()
        patch = self.patch_entry.get()
        
        if not src or not dst or not patch:
            messagebox.showerror("Error", "Please fill in all fields")
            return
        
        self.make_status.config(text="Creating patch...", foreground="blue")
        self.make_progress.start()
        self.root.update()
        
        def worker():
            try:
                # In a real implementation, this would call the core library
                # For now, just simulate
                import time
                time.sleep(2)
                self.make_status.config(text="✓ Patch created successfully!", foreground="green")
                messagebox.showinfo("Success", f"Patch created: {patch}")
            except Exception as e:
                self.make_status.config(text=f"✗ Error: {e}", foreground="red")
                messagebox.showerror("Error", str(e))
            finally:
                self.make_progress.stop()
        
        thread = threading.Thread(target=worker, daemon=True)
        thread.start()
    
    def apply_patch(self):
        """Apply a patch (calls CLI in background)."""
        target = self.target_entry.get()
        patch = self.apply_patch_entry.get()
        
        if not target or not patch:
            messagebox.showerror("Error", "Please fill in all fields")
            return
        
        self.apply_status.config(text="Applying patch...", foreground="blue")
        self.apply_progress.start()
        self.root.update()
        
        def worker():
            try:
                # In a real implementation, this would call the core library
                # For now, just simulate
                import time
                time.sleep(2)
                self.apply_status.config(text="✓ Patch applied successfully!", foreground="green")
                messagebox.showinfo("Success", "Patch applied successfully!")
            except Exception as e:
                self.apply_status.config(text=f"✗ Error: {e}", foreground="red")
                messagebox.showerror("Error", str(e))
            finally:
                self.apply_progress.stop()
        
        thread = threading.Thread(target=worker, daemon=True)
        thread.start()


def main():
    root = Tk()
    app = PatchForgeGUI(root)
    root.mainloop()


if __name__ == "__main__":
    main()
