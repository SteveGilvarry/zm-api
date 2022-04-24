import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitor_Status_Status } from './monitor-status-status.enum';

@InputType()
export class EnumMonitor_Status_StatusFieldUpdateOperationsInput {

    @Field(() => Monitor_Status_Status, {nullable:true})
    set?: keyof typeof Monitor_Status_Status;
}
