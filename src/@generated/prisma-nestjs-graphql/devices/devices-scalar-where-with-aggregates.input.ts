import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { StringWithAggregatesFilter } from '../prisma/string-with-aggregates-filter.input';
import { EnumDevices_TypeWithAggregatesFilter } from '../prisma/enum-devices-type-with-aggregates-filter.input';

@InputType()
export class DevicesScalarWhereWithAggregatesInput {

    @Field(() => [DevicesScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<DevicesScalarWhereWithAggregatesInput>;

    @Field(() => [DevicesScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<DevicesScalarWhereWithAggregatesInput>;

    @Field(() => [DevicesScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<DevicesScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Id?: IntWithAggregatesFilter;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    Name?: StringWithAggregatesFilter;

    @Field(() => EnumDevices_TypeWithAggregatesFilter, {nullable:true})
    Type?: EnumDevices_TypeWithAggregatesFilter;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    KeyString?: StringWithAggregatesFilter;
}
