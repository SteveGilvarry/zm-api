import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlsWhereInput } from './controls-where.input';
import { Type } from 'class-transformer';
import { ControlsOrderByWithRelationInput } from './controls-order-by-with-relation.input';
import { ControlsWhereUniqueInput } from './controls-where-unique.input';
import { Int } from '@nestjs/graphql';
import { ControlsScalarFieldEnum } from './controls-scalar-field.enum';

@ArgsType()
export class FindFirstControlsArgs {

    @Field(() => ControlsWhereInput, {nullable:true})
    @Type(() => ControlsWhereInput)
    where?: ControlsWhereInput;

    @Field(() => [ControlsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<ControlsOrderByWithRelationInput>;

    @Field(() => ControlsWhereUniqueInput, {nullable:true})
    cursor?: ControlsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [ControlsScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof ControlsScalarFieldEnum>;
}
