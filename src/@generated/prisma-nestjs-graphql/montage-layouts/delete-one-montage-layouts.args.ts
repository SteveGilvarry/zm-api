import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MontageLayoutsWhereUniqueInput } from './montage-layouts-where-unique.input';

@ArgsType()
export class DeleteOneMontageLayoutsArgs {

    @Field(() => MontageLayoutsWhereUniqueInput, {nullable:false})
    where!: MontageLayoutsWhereUniqueInput;
}
