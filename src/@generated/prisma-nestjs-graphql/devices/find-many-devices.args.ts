import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { DevicesWhereInput } from './devices-where.input';
import { Type } from 'class-transformer';
import { DevicesOrderByWithRelationInput } from './devices-order-by-with-relation.input';
import { DevicesWhereUniqueInput } from './devices-where-unique.input';
import { Int } from '@nestjs/graphql';
import { DevicesScalarFieldEnum } from './devices-scalar-field.enum';

@ArgsType()
export class FindManyDevicesArgs {

    @Field(() => DevicesWhereInput, {nullable:true})
    @Type(() => DevicesWhereInput)
    where?: DevicesWhereInput;

    @Field(() => [DevicesOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<DevicesOrderByWithRelationInput>;

    @Field(() => DevicesWhereUniqueInput, {nullable:true})
    cursor?: DevicesWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [DevicesScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof DevicesScalarFieldEnum>;
}
