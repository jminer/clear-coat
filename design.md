
The wrapper does reference counting of controls. Every wrapper handle holds a reference to a control. If the control has a parent, the parent holds a reference. Reference counting prevents dangling pointers that could happen using IUP directly. Additionally, if a control is destroyed using the IUP API directly, the wrapper will know and panic rather than using a dangling pointer.

Since it is possible to drop wrapper structs without destroying the control they are wrapping, they can't really store anything. Callbacks and such need to be stored in statics.