import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitor_Status_Status } from './monitor-status-status.enum';

@InputType()
export class NestedEnumMonitor_Status_StatusFilter {

    @Field(() => Monitor_Status_Status, {nullable:true})
    equals?: keyof typeof Monitor_Status_Status;

    @Field(() => [Monitor_Status_Status], {nullable:true})
    in?: Array<keyof typeof Monitor_Status_Status>;

    @Field(() => [Monitor_Status_Status], {nullable:true})
    notIn?: Array<keyof typeof Monitor_Status_Status>;

    @Field(() => NestedEnumMonitor_Status_StatusFilter, {nullable:true})
    not?: NestedEnumMonitor_Status_StatusFilter;
}
