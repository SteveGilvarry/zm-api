import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Servers_Status } from './servers-status.enum';

@InputType()
export class EnumServers_StatusFieldUpdateOperationsInput {

    @Field(() => Servers_Status, {nullable:true})
    set?: keyof typeof Servers_Status;
}
