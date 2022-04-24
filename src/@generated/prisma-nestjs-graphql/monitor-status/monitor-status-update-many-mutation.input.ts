import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFieldUpdateOperationsInput } from '../prisma/int-field-update-operations.input';
import { EnumMonitor_Status_StatusFieldUpdateOperationsInput } from '../prisma/enum-monitor-status-status-field-update-operations.input';
import { DecimalFieldUpdateOperationsInput } from '../prisma/decimal-field-update-operations.input';
import { NullableBigIntFieldUpdateOperationsInput } from '../prisma/nullable-big-int-field-update-operations.input';

@InputType()
export class Monitor_StatusUpdateManyMutationInput {

    @Field(() => IntFieldUpdateOperationsInput, {nullable:true})
    MonitorId?: IntFieldUpdateOperationsInput;

    @Field(() => EnumMonitor_Status_StatusFieldUpdateOperationsInput, {nullable:true})
    Status?: EnumMonitor_Status_StatusFieldUpdateOperationsInput;

    @Field(() => DecimalFieldUpdateOperationsInput, {nullable:true})
    CaptureFPS?: DecimalFieldUpdateOperationsInput;

    @Field(() => DecimalFieldUpdateOperationsInput, {nullable:true})
    AnalysisFPS?: DecimalFieldUpdateOperationsInput;

    @Field(() => IntFieldUpdateOperationsInput, {nullable:true})
    CaptureBandwidth?: IntFieldUpdateOperationsInput;

    @Field(() => NullableBigIntFieldUpdateOperationsInput, {nullable:true})
    DayEventDiskSpace?: NullableBigIntFieldUpdateOperationsInput;
}
