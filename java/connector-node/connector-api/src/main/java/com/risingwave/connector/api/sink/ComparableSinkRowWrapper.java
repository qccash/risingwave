package com.risingwave.connector.api.sink;

import com.risingwave.proto.Data;

public class ComparableSinkRowWrapper implements SinkRow {
    public final Object[] values;
    public final Data.Op op;
    public final int internalIndex;

    public ComparableSinkRowWrapper(int internalIndex, Data.Op op, Object... value) {
        this.internalIndex = internalIndex;
        this.op = op;
        this.values = value;
    }

    @Override
    public Object get(int index) {
        return values[index];
    }

    @Override
    public Data.Op getOp() {
        return op;
    }

    @Override
    public int size() {
        return values.length;
    }

    public int getInternalIndex() {
        return internalIndex;
    }

    public static ComparableSinkRowWrapper from(SinkRow row, int internalIndex) {
        Object[] values = new Object[row.size()];
        for (int i = 0; i < row.size(); i++) {
            values[i] = row.get(i);
        }
        return new ComparableSinkRowWrapper(internalIndex, row.getOp(), values);
    }
}
