import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsWhereInput } from './control-presets-where.input';
import { ControlPresetsOrderByWithRelationInput } from './control-presets-order-by-with-relation.input';
import { ControlPresetsWhereUniqueInput } from './control-presets-where-unique.input';
import { Int } from '@nestjs/graphql';
import { ControlPresetsScalarFieldEnum } from './control-presets-scalar-field.enum';

@ArgsType()
export class FindManyControlPresetsArgs {

    @Field(() => ControlPresetsWhereInput, {nullable:true})
    where?: ControlPresetsWhereInput;

    @Field(() => [ControlPresetsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<ControlPresetsOrderByWithRelationInput>;

    @Field(() => ControlPresetsWhereUniqueInput, {nullable:true})
    cursor?: ControlPresetsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [ControlPresetsScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof ControlPresetsScalarFieldEnum>;
}
