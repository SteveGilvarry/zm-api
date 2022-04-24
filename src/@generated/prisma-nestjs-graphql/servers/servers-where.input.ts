import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringNullableFilter } from '../prisma/string-nullable-filter.input';
import { IntNullableFilter } from '../prisma/int-nullable-filter.input';
import { StringFilter } from '../prisma/string-filter.input';
import { EnumServers_StatusFilter } from '../prisma/enum-servers-status-filter.input';
import { DecimalNullableFilter } from '../prisma/decimal-nullable-filter.input';
import { BigIntNullableFilter } from '../prisma/big-int-nullable-filter.input';
import { BoolFilter } from '../prisma/bool-filter.input';

@InputType()
export class ServersWhereInput {

    @Field(() => [ServersWhereInput], {nullable:true})
    AND?: Array<ServersWhereInput>;

    @Field(() => [ServersWhereInput], {nullable:true})
    OR?: Array<ServersWhereInput>;

    @Field(() => [ServersWhereInput], {nullable:true})
    NOT?: Array<ServersWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    Id?: IntFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Protocol?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    Hostname?: StringNullableFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    Port?: IntNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    PathToIndex?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    PathToZMS?: StringNullableFilter;

    @Field(() => StringNullableFilter, {nullable:true})
    PathToApi?: StringNullableFilter;

    @Field(() => StringFilter, {nullable:true})
    Name?: StringFilter;

    @Field(() => IntNullableFilter, {nullable:true})
    State_Id?: IntNullableFilter;

    @Field(() => EnumServers_StatusFilter, {nullable:true})
    Status?: EnumServers_StatusFilter;

    @Field(() => DecimalNullableFilter, {nullable:true})
    CpuLoad?: DecimalNullableFilter;

    @Field(() => BigIntNullableFilter, {nullable:true})
    TotalMem?: BigIntNullableFilter;

    @Field(() => BigIntNullableFilter, {nullable:true})
    FreeMem?: BigIntNullableFilter;

    @Field(() => BigIntNullableFilter, {nullable:true})
    TotalSwap?: BigIntNullableFilter;

    @Field(() => BigIntNullableFilter, {nullable:true})
    FreeSwap?: BigIntNullableFilter;

    @Field(() => BoolFilter, {nullable:true})
    zmstats?: BoolFilter;

    @Field(() => BoolFilter, {nullable:true})
    zmaudit?: BoolFilter;

    @Field(() => BoolFilter, {nullable:true})
    zmtrigger?: BoolFilter;

    @Field(() => BoolFilter, {nullable:true})
    zmeventnotification?: BoolFilter;
}
