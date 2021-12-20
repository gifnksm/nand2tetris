use super::*;

#[derive(Debug, Clone, Default)]
pub(super) struct Emitter(Vec<Command>);

impl Emitter {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn into_commands(self) -> Vec<Command> {
        self.0
    }

    fn emit(&mut self, command: Command) {
        self.0.push(command);
    }

    pub(super) fn emit_add(&mut self) {
        self.emit(Command::Add);
    }

    pub(super) fn emit_sub(&mut self) {
        self.emit(Command::Sub);
    }

    pub(super) fn emit_neg(&mut self) {
        self.emit(Command::Neg);
    }

    pub(super) fn emit_eq(&mut self) {
        self.emit(Command::Eq);
    }

    pub(super) fn emit_gt(&mut self) {
        self.emit(Command::Gt);
    }

    pub(super) fn emit_lt(&mut self) {
        self.emit(Command::Lt);
    }

    pub(super) fn emit_and(&mut self) {
        self.emit(Command::And);
    }

    pub(super) fn emit_or(&mut self) {
        self.emit(Command::Or);
    }

    pub(super) fn emit_not(&mut self) {
        self.emit(Command::Not);
    }

    pub(super) fn emit_push<T>(&mut self, seg: Segment, slot: T)
    where
        T: TryInto<u16>,
        <T as TryInto<u16>>::Error: fmt::Debug,
    {
        self.emit(Command::Push(seg, slot.try_into().unwrap()));
    }

    pub(super) fn emit_push_constant<T>(&mut self, value: T)
    where
        T: TryInto<u16>,
        <T as TryInto<u16>>::Error: fmt::Debug,
    {
        self.emit_push(Segment::Constant, value);
    }

    pub(super) fn emit_push_this_addr(&mut self) {
        self.emit_push(Segment::Pointer, 0);
    }

    pub(super) fn emit_push_that_value(&mut self) {
        self.emit_push(Segment::That, 0);
    }

    pub(super) fn emit_pop<T>(&mut self, seg: Segment, slot: T)
    where
        T: TryInto<u16>,
        <T as TryInto<u16>>::Error: fmt::Debug,
    {
        self.emit(Command::Pop(seg, slot.try_into().unwrap()));
    }

    pub(super) fn emit_pop_this_addr(&mut self) {
        self.emit_pop(Segment::Pointer, 0);
    }

    pub(super) fn emit_pop_that_addr(&mut self) {
        self.emit_pop(Segment::Pointer, 1);
    }

    pub(super) fn emit_label(&mut self, id: BbId) {
        self.emit(Command::Label(id.to_string().parse().unwrap()));
    }

    pub(super) fn emit_goto(&mut self, id: BbId) {
        self.emit(Command::Goto(id.to_string().parse().unwrap()));
    }

    pub(super) fn emit_if_goto(&mut self, id: BbId) {
        self.emit(Command::IfGoto(id.to_string().parse().unwrap()));
    }

    pub(super) fn emit_function<T>(
        &mut self,
        class_name: impl fmt::Display,
        sub_name: impl fmt::Display,
        num_vars: T,
    ) where
        T: TryInto<u8>,
        <T as TryInto<u8>>::Error: fmt::Debug,
    {
        self.emit(Command::Function(
            format!("{}.{}", class_name, sub_name).parse().unwrap(),
            num_vars.try_into().unwrap(),
        ));
    }

    pub(super) fn emit_call<T>(
        &mut self,
        class_name: impl fmt::Display,
        sub_name: impl fmt::Display,
        arity: T,
    ) where
        T: TryInto<u8>,
        <T as TryInto<u8>>::Error: fmt::Debug,
    {
        self.emit(Command::Call(
            format!("{}.{}", class_name, sub_name).parse().unwrap(),
            arity.try_into().unwrap(),
        ));
    }

    pub(super) fn emit_call_with_args(
        &mut self,
        class_name: impl fmt::Display,
        sub_name: impl fmt::Display,
        has_receiver: bool,
        args: &[WithLoc<TypedExpression>],
        p: &EmitVmBbParam,
    ) {
        for arg in args {
            arg.emit_vm(p, self);
        }
        let mut arity = args.len();
        if has_receiver {
            arity += 1;
        }
        self.emit_call(class_name, sub_name, arity);
    }

    pub(super) fn emit_return(&mut self) {
        self.emit(Command::Return);
    }
}
