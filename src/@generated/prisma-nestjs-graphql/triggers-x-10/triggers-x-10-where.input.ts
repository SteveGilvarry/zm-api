import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringNullableFilter } from '../prisma/string-nullable-filter.input';

@InputType()
export class TriggersX10WhereInput {

    @Field(() => [TriggersX10WhereInput], {nullable:true})
    AND?: Array<TriggersX10WhereInput>;

    @Field(() => [TriggersX10WhereInput], {nullable:true})
    OR?: Array<TriggersX10WhereInput>;

    @Field(() => [TriggersX10WhereInput], {nullable:true})
    NOT?: Array<TriggersX10WhereInput>;

    @Field(() => IntFilter, {nullable:true})
    MonitorId?: IntFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Activation?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    AlarmInput?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    AlarmOutput?: StringNullableFilter;
}
