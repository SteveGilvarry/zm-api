import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class TriggersX10OrderByWithRelationInput {

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Activation?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AlarmInput?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    AlarmOutput?: keyof typeof SortOrder;
}
