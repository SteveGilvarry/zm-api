import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { BigIntNullableFilter } from '../prisma/big-int-nullable-filter.input';

@InputType()
export class Events_ArchivedWhereInput {

    @Field(() => [Events_ArchivedWhereInput], {nullable:true})
    AND?: Array<Events_ArchivedWhereInput>;

    @Field(() => [Events_ArchivedWhereInput], {nullable:true})
    OR?: Array<Events_ArchivedWhereInput>;

    @Field(() => [Events_ArchivedWhereInput], {nullable:true})
    NOT?: Array<Events_ArchivedWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    EventId?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    MonitorId?: IntFilter;

    @Field(() => BigIntNullableFilter, {nullable:true})
    DiskSpace?: BigIntNullableFilter;
}
