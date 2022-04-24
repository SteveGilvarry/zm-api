import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_Importance } from '../monitors/monitors-importance.enum';

@InputType()
export class NullableEnumMonitors_ImportanceFieldUpdateOperationsInput {

    @Field(() => Monitors_Importance, {nullable:true})
    set?: keyof typeof Monitors_Importance;
}
