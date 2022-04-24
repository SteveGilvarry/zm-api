import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFieldUpdateOperationsInput } from '../prisma/int-field-update-operations.input';

@InputType()
export class Groups_MonitorsUpdateInput {

    @Field(() => IntFieldUpdateOperationsInput, {nullable:true})
    GroupId?: IntFieldUpdateOperationsInput;

    @Field(() => IntFieldUpdateOperationsInput, {nullable:true})
    MonitorId?: IntFieldUpdateOperationsInput;
}
